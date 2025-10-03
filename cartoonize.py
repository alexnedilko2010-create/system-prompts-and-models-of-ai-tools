import cv2
import numpy as np
from PIL import Image
from pathlib import Path


def cartoonize(image_bgr: np.ndarray) -> np.ndarray:
    # Edge-preserving smoothing
    filtered = cv2.bilateralFilter(image_bgr, d=9, sigmaColor=150, sigmaSpace=150)

    # Edge map
    gray = cv2.cvtColor(filtered, cv2.COLOR_BGR2GRAY)
    blur = cv2.medianBlur(gray, 7)
    edges = cv2.adaptiveThreshold(blur, 255, cv2.ADAPTIVE_THRESH_MEAN_C, cv2.THRESH_BINARY, 9, 2)
    edges = cv2.cvtColor(edges, cv2.COLOR_GRAY2BGR)

    # Color quantization (reduce palette)
    data = filtered.reshape((-1, 3)).astype(np.float32)
    K = 8
    criteria = (cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER, 20, 0.5)
    _, labels, centers = cv2.kmeans(data, K, None, criteria, 2, cv2.KMEANS_PP_CENTERS)
    centers = np.uint8(centers)
    quant = centers[labels.flatten()].reshape(filtered.shape)

    # Warm tone like reference image
    warm = quant.astype(np.float32)
    warm[:, :, 2] = np.clip(warm[:, :, 2] * 1.15 + 10, 0, 255)  # boost R
    warm[:, :, 1] = np.clip(warm[:, :, 1] * 1.05 + 5, 0, 255)   # boost G
    warm[:, :, 0] = np.clip(warm[:, :, 0] * 0.95, 0, 255)       # reduce B
    warm = warm.astype(np.uint8)

    # Combine with edges
    cartoon = cv2.bitwise_and(warm, edges)
    return cartoon


def main():
    content_path = Path('assets/input/content.jpg')
    output_path = Path('assets/output/stylized.jpg')
    output_path.parent.mkdir(parents=True, exist_ok=True)

    img = cv2.imread(str(content_path))
    if img is None:
        raise FileNotFoundError(f"Cannot read {content_path}")
    result = cartoonize(img)
    cv2.imwrite(str(output_path), result)
    print(f"Saved {output_path}")


if __name__ == '__main__':
    main()

