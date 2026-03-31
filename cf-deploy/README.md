# pathscale-be — Cloudflare Containers deploy

Deploys `pathscale_be` as a Cloudflare Container backed by an R2 bucket (via tigrisfs FUSE mount).

## Prerequisites

- `wrangler` CLI installed and authenticated
- An R2 bucket created in your Cloudflare account
- Tigris/R2 S3-compatible credentials (Access Key ID + Secret)

## First-time setup

### 1. Install dependencies

```sh
cd cf-deploy
npm install
```

### 2. Set wrangler vars

Uncomment and fill in the `[vars]` block in `wrangler.toml`:

```toml
[vars]
HONEY_ID_ADDR = "wss://auth.honey.id/"
DB_PATH = "/mnt/r2/db"
R2_ACCOUNT_ID = "<your-cloudflare-account-id>"
R2_BUCKET_NAME = "<your-r2-bucket-name>"
R2_PREFIX = "<optional-key-prefix>"   # e.g. "pathscale-prod"
ADMIN_PUB_ID = "<admin-user-uuid>"    # optional
```

### 3. Set secrets

```sh
wrangler secret put HONEY_ID_APP_PUBLIC_ID
wrangler secret put HONEY_ID_AUTH_API_KEY
wrangler secret put AWS_ACCESS_KEY_ID      # R2/Tigris access key
wrangler secret put AWS_SECRET_ACCESS_KEY  # R2/Tigris secret key
wrangler secret put TG_BOT_TOKEN           # optional — Telegram bot
```

### 4. Deploy

```sh
wrangler deploy
```

## Local development

Create `cf-deploy/.dev.vars` (gitignored) with all vars and secrets:

```ini
HONEY_ID_ADDR=wss://auth.honey.id/
DB_PATH=/mnt/r2/db
R2_ACCOUNT_ID=<your-account-id>
R2_BUCKET_NAME=<your-bucket>
R2_PREFIX=pathscale-dev
ADMIN_PUB_ID=<uuid>
HONEY_ID_APP_PUBLIC_ID=<uuid>
HONEY_ID_AUTH_API_KEY=<key>
AWS_ACCESS_KEY_ID=<key>
AWS_SECRET_ACCESS_KEY=<secret>
TG_BOT_TOKEN=<token>
```

Then:

```sh
wrangler dev
```

## How it works

On container start, `entrypoint.cf.sh` (at repo root):
1. Mounts the R2 bucket at `/mnt/r2` using tigrisfs
2. Generates `/tmp/config.toml` from env vars
3. Execs `pathscale_be` with `CONFIG=/tmp/config.toml`

The container image is built from `Dockerfile.cf` at the repo root.
