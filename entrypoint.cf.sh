#!/bin/sh
set -e

# Mount R2/Tigris bucket via tigrisfs
mkdir -p /mnt/r2

R2_ENDPOINT="https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com"
BUCKET_TARGET="${R2_BUCKET_NAME}${R2_PREFIX:+:${R2_PREFIX}}"
echo "Mounting ${BUCKET_TARGET} at /mnt/r2..."
/usr/local/bin/tigrisfs --endpoint "${R2_ENDPOINT}" -f "${BUCKET_TARGET}" /mnt/r2 &
sleep 3

mkdir -p /tmp/logs

cat > /tmp/config.toml << EOF
[database]
path = "${DB_PATH:-/mnt/r2/db}"

[log]
level = "info"
folder = "/tmp/logs"

[server]
name = "pathscale_be"
address = "0.0.0.0:8080"
insecure = true

[honey_id]
addr = "${HONEY_ID_ADDR:-wss://auth.honey.id/}"
app_public_id = "${HONEY_ID_APP_PUBLIC_ID}"
auth_api_key = "${HONEY_ID_AUTH_API_KEY}"
EOF

if [ -n "${TG_BOT_TOKEN}" ]; then
  cat >> /tmp/config.toml << EOFTG

[tg_bot]
enabled = true
token = "${TG_BOT_TOKEN}"
EOFTG
fi

if [ -n "${ADMIN_PUB_ID}" ]; then
  cat >> /tmp/config.toml << EOFADMIN

[user]
admin_pub_id = "${ADMIN_PUB_ID}"
EOFADMIN
fi

exec env CONFIG=/tmp/config.toml /usr/local/bin/pathscale_be
