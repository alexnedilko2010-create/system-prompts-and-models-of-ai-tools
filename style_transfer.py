import argparse
from pathlib import Path
from typing import Tuple, List

import torch
import torch.nn as nn
import torch.optim as optim
from PIL import Image
from torchvision import models, transforms
from tqdm import tqdm


# ---------------------------------------------
# Utilities
# ---------------------------------------------

def load_image(image_path: Path, max_size: int = 768) -> torch.Tensor:
    """
    Loads an image and returns a tensor normalized for VGG.
    """
    if not image_path.exists():
        raise FileNotFoundError(f"Image not found: {image_path}")

    image = Image.open(str(image_path)).convert("RGB")

    # Resize keeping aspect ratio
    size = max(image.size)
    if size > max_size:
        scale = max_size / size
        new_w = int(image.size[0] * scale)
        new_h = int(image.size[1] * scale)
        image = image.resize((new_w, new_h), Image.LANCZOS)

    vgg_normalize = transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225])
    to_tensor = transforms.Compose([transforms.ToTensor(), vgg_normalize])
    return to_tensor(image).unsqueeze(0)


def tensor_to_image(tensor: torch.Tensor) -> Image.Image:
    """
    Converts VGG-normalized tensor to PIL image.
    """
    mean = torch.tensor([0.485, 0.456, 0.406], device=tensor.device).view(1, 3, 1, 1)
    std = torch.tensor([0.229, 0.224, 0.225], device=tensor.device).view(1, 3, 1, 1)
    image = tensor.mul(std).add(mean)
    image = image.clamp(0, 1)
    image = image.squeeze(0).permute(1, 2, 0).detach().cpu().numpy()
    image = (image * 255).round().astype("uint8")
    return Image.fromarray(image)


class VGGFeatures(nn.Module):
    """
    Extracts intermediate VGG19 features for style and content losses.
    """

    def __init__(self, content_layers: List[str], style_layers: List[str]):
        super().__init__()
        vgg = models.vgg19(weights=models.VGG19_Weights.IMAGENET1K_V1).features
        # Freeze
        for param in vgg.parameters():
            param.requires_grad = False

        self.model = vgg
        self.content_layers = set(content_layers)
        self.style_layers = set(style_layers)

        self.layer_name_mapping = {}
        block = 1
        conv = 0
        for idx, layer in enumerate(self.model):
            if isinstance(layer, nn.Conv2d):
                conv += 1
                name = f"conv{block}_{conv}"
            elif isinstance(layer, nn.ReLU):
                name = f"relu{block}_{conv}"
                # use inplace=False to keep gradients stable
                self.model[idx] = nn.ReLU(inplace=False)
            elif isinstance(layer, nn.MaxPool2d):
                name = f"pool{block}"
                block += 1
                conv = 0
            else:
                name = f"layer_{idx}"
            self.layer_name_mapping[idx] = name

    def forward(self, x: torch.Tensor) -> Tuple[dict, dict]:
        content_feats = {}
        style_feats = {}
        for idx, layer in enumerate(self.model):
            x = layer(x)
            name = self.layer_name_mapping[idx]
            if name in self.content_layers:
                content_feats[name] = x
            if name in self.style_layers:
                style_feats[name] = x
        return content_feats, style_feats


def gram_matrix(tensor: torch.Tensor) -> torch.Tensor:
    b, c, h, w = tensor.size()
    features = tensor.view(b * c, h * w)
    gram = features @ features.t()
    return gram / (c * h * w)


def style_transfer(
    content_img: torch.Tensor,
    style_img: torch.Tensor,
    steps: int = 400,
    content_weight: float = 1.0,
    style_weight: float = 10_000.0,
    tv_weight: float = 0.0,
    lr: float = 0.05,
    device: str = "cuda"
) -> torch.Tensor:
    device = torch.device(device if torch.cuda.is_available() and device == "cuda" else "cpu")
    content_img = content_img.to(device)
    style_img = style_img.to(device)

    # Match spatial size of style to content for stability
    if style_img.shape[-2:] != content_img.shape[-2:]:
        style_img = torch.nn.functional.interpolate(style_img, size=content_img.shape[-2:], mode="bilinear", align_corners=False)

    content_layers = ["relu4_2"]
    style_layers = ["relu1_1", "relu2_1", "relu3_1", "relu4_1", "relu5_1"]

    extractor = VGGFeatures(content_layers, style_layers).to(device).eval()

    with torch.no_grad():
        content_targets, _ = extractor(content_img)
        _, style_feats = extractor(style_img)
        style_targets = {name: gram_matrix(feat) for name, feat in style_feats.items()}

    generated = content_img.clone().requires_grad_(True)
    optimizer = optim.Adam([generated], lr=lr)

    pbar = tqdm(range(steps), desc="Optimizing", ncols=100)
    for _ in pbar:
        optimizer.zero_grad()
        gen_content, gen_style = extractor(generated)

        # Content loss
        content_loss = 0.0
        for name in content_layers:
            content_loss = content_loss + torch.nn.functional.mse_loss(gen_content[name], content_targets[name])

        # Style loss
        style_loss = 0.0
        for name in style_layers:
            G = gram_matrix(gen_style[name])
            A = style_targets[name]
            style_loss = style_loss + torch.nn.functional.mse_loss(G, A)

        # Total variation loss (optional smoothness)
        tv_loss = 0.0
        if tv_weight > 0:
            tv_loss = (
                torch.mean(torch.abs(generated[:, :, :, :-1] - generated[:, :, :, 1:])) +
                torch.mean(torch.abs(generated[:, :, :-1, :] - generated[:, :, 1:, :]))
            )

        loss = content_weight * content_loss + style_weight * style_loss + tv_weight * tv_loss
        loss.backward()
        optimizer.step()

        pbar.set_postfix({"loss": f"{loss.item():.2f}", "c": f"{content_loss.item():.2f}", "s": f"{style_loss.item():.2f}"})

        # Clamp to VGG input range in normalized space is tricky; clamp in image space via de-normalization in save step

    return generated.detach().cpu()


def main():
    parser = argparse.ArgumentParser(description="Neural style transfer: make image 1 look like the style of image 2")
    parser.add_argument("--content", type=str, required=False, default="assets/input/content.jpg", help="Path to content (photo 1)")
    parser.add_argument("--style", type=str, required=False, default="assets/style/style.jpg", help="Path to style (image 2)")
    parser.add_argument("--output", type=str, required=False, default="assets/output/stylized.jpg", help="Output path")
    parser.add_argument("--steps", type=int, default=400)
    parser.add_argument("--content_weight", type=float, default=1.0)
    parser.add_argument("--style_weight", type=float, default=10000.0)
    parser.add_argument("--tv_weight", type=float, default=0.0)
    parser.add_argument("--lr", type=float, default=0.05)
    parser.add_argument("--max_size", type=int, default=768, help="Max dimension for processing")
    parser.add_argument("--device", type=str, default="cuda", choices=["cuda", "cpu"], help="Prefer GPU if available")
    args = parser.parse_args()

    content_path = Path(args.content)
    style_path = Path(args.style)
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    content_img = load_image(content_path, max_size=args.max_size)
    style_img = load_image(style_path, max_size=args.max_size)

    result = style_transfer(
        content_img=content_img,
        style_img=style_img,
        steps=args.steps,
        content_weight=args.content_weight,
        style_weight=args.style_weight,
        tv_weight=args.tv_weight,
        lr=args.lr,
        device=args.device,
    )

    pil = tensor_to_image(result)
    pil.save(str(output_path))
    print(f"Saved stylized image to {output_path}")


if __name__ == "__main__":
    main()

