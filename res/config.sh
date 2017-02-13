# From address (linked to PGP key)
FROM_ADDR=mail@example.com

# Address of the hetzner robot
TO_ADDR=robot@robot.first-ns.de

# SMTP data
SMTP_HOST=smtp.exaple.com
SMTP_PORT=465
SMTP_USERNAME=user
SMTP_PASSWORD=pass

# ID of the PGP key with which the message to hetzner will be signed
PGP_KEY=0000

# Domain which is to be managed
DOMAIN=example.com

# Hetzner user
HETZNER_USER=user

# Server listening address
SERVER_ADDR=0.0.0.0:0

# HTTP Basic auth
HTTP_AUTH_USER=user
HTTP_AUTH_PASSWORD=pass

# Header from which the real ip of the client is to be read
IP_HEADER=X-Real-IP
