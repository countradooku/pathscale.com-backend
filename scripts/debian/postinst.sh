#!/bin/bash
# Reload the systemd daemon to apply changes
systemctl daemon-reload

# Create directories if they don't exist
mkdir -p /var/log/pathscale_be
mkdir -p /var/lib/pathscale_be/db/1.0

# Set permissions on bin
chown deploy:deploy /usr/local/pathscale_be/pathscale_be
setcap CAP_NET_BIND_SERVICE=+eip /usr/local/pathscale_be/pathscale_be

# Set permissions on systemd service
chown deploy:deploy /etc/systemd/system/pathscale_be.service

# Set permissions on config directory and file
chown deploy:deploy /etc/pathscale_be
chmod 755 /etc/pathscale_be
chown deploy:deploy /etc/pathscale_be/config.deploy.toml
chmod 644 /etc/pathscale_be/config.deploy.toml

# Set permissions on log directory
chown deploy:deploy -R /var/log/pathscale_be
chmod 755 /var/log/pathscale_be


# Set permissions on data directory
chown deploy:deploy -R /var/lib/pathscale_be
chmod 755 /var/lib/pathscale_be
chown deploy:deploy -R /var/lib/pathscale_be/db/1.0
chmod 755 /var/lib/pathscale_be/db/1.0

#certs
chown deploy:deploy -R /etc/letsencrypt/
