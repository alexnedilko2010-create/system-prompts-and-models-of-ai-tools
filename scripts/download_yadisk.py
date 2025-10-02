import sys
import json
import urllib.parse
import urllib.request
import ssl
from pathlib import Path


def get_direct_download_url(public_url: str) -> str:
    api = (
        "https://cloud-api.yandex.net/v1/disk/public/resources/download?public_key="
        + urllib.parse.quote(public_url, safe="")
    )
    ctx = ssl.create_default_context()
    with urllib.request.urlopen(api, context=ctx) as resp:
        data = json.loads(resp.read().decode("utf-8"))
    href = data.get("href")
    if not href:
        raise RuntimeError(f"No href in API response: {data}")
    return href


def download_to(public_url: str, out_path: Path) -> None:
    href = get_direct_download_url(public_url)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    ctx = ssl.create_default_context()
    with urllib.request.urlopen(href, context=ctx) as resp, open(out_path, "wb") as f:
        while True:
            chunk = resp.read(1024 * 1024)
            if not chunk:
                break
            f.write(chunk)


def main() -> int:
    if len(sys.argv) < 3:
        print("Usage: download_yadisk.py <public_url> <output_path>", file=sys.stderr)
        return 2
    public_url = sys.argv[1]
    out_path = Path(sys.argv[2])
    try:
        download_to(public_url, out_path)
    except Exception as exc:
        print(f"Download failed: {exc}", file=sys.stderr)
        return 1
    print(str(out_path.resolve()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

