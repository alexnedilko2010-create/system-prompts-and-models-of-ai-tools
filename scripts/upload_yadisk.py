import sys
import json
import urllib.parse
import urllib.request
import ssl
from pathlib import Path


def get_upload_url(path: str, token: str) -> str:
    params = urllib.parse.urlencode({"path": path, "overwrite": "true"})
    url = f"https://cloud-api.yandex.net/v1/disk/resources/upload?{params}"
    req = urllib.request.Request(url)
    req.add_header("Authorization", f"OAuth {token}")
    ctx = ssl.create_default_context()
    with urllib.request.urlopen(req, context=ctx) as resp:
        data = json.loads(resp.read().decode("utf-8"))
    href = data.get("href")
    if not href:
        raise RuntimeError(f"No href in response: {data}")
    return href


def upload_file(local_path: Path, remote_path: str, token: str) -> None:
    href = get_upload_url(remote_path, token)
    ctx = ssl.create_default_context()
    with open(local_path, "rb") as f:
        data = f.read()
    req = urllib.request.Request(href, data=data, method="PUT")
    with urllib.request.urlopen(req, context=ctx) as resp:
        resp.read()


def publish_and_get_public_url(path: str, token: str) -> str:
    url = f"https://cloud-api.yandex.net/v1/disk/resources/publish?path={urllib.parse.quote(path, safe='')}"
    req = urllib.request.Request(url, method="PUT")
    req.add_header("Authorization", f"OAuth {token}")
    ctx = ssl.create_default_context()
    with urllib.request.urlopen(req, context=ctx) as resp:
        resp.read()

    # Read meta to get public_url
    meta = f"https://cloud-api.yandex.net/v1/disk/resources?path={urllib.parse.quote(path, safe='')}"
    meta_req = urllib.request.Request(meta)
    meta_req.add_header("Authorization", f"OAuth {token}")
    with urllib.request.urlopen(meta_req, context=ctx) as resp:
        data = json.loads(resp.read().decode("utf-8"))
    public_url = data.get("public_url")
    if not public_url:
        raise RuntimeError("Failed to obtain public_url")
    return public_url


def main() -> int:
    if len(sys.argv) < 4:
        print("Usage: upload_yadisk.py <local_file> <remote_path> <oauth_token>", file=sys.stderr)
        return 2
    local_file = Path(sys.argv[1])
    remote_path = sys.argv[2]
    token = sys.argv[3]
    if not local_file.exists():
        print(f"File not found: {local_file}", file=sys.stderr)
        return 1
    try:
        upload_file(local_file, remote_path, token)
        url = publish_and_get_public_url(remote_path, token)
    except Exception as exc:
        print(f"Upload failed: {exc}", file=sys.stderr)
        return 1
    print(url)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

