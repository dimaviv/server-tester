#!/bin/bash
set -euo pipefail

# EC2 setup script for Amazon Linux 2023 or Ubuntu 22.04 (t2.micro free tier)
# Run as root or with sudo: sudo bash ec2-setup.sh

echo "=== Installing Docker ==="
if command -v apt-get &>/dev/null; then
    apt-get update
    apt-get install -y docker.io docker-compose-plugin git
    systemctl enable docker
    systemctl start docker
    usermod -aG docker ubuntu
elif command -v yum &>/dev/null; then
    yum update -y
    yum install -y docker git
    systemctl enable docker
    systemctl start docker
    usermod -aG docker ec2-user
    DOCKER_CONFIG=/usr/local/lib/docker
    mkdir -p "$DOCKER_CONFIG/cli-plugins"
    curl -SL https://github.com/docker/compose/releases/latest/download/docker-compose-linux-x86_64 \
        -o "$DOCKER_CONFIG/cli-plugins/docker-compose"
    chmod +x "$DOCKER_CONFIG/cli-plugins/docker-compose"
fi

echo "=== Cloning repository ==="
cd /opt
if [ -d server-tester ]; then
    cd server-tester && git pull
else
    git clone "$1" server-tester
    cd server-tester
fi

echo "=== Building and starting ==="
docker compose up -d --build

PUBLIC_IP=$(curl -s http://169.254.169.254/latest/meta-data/public-ipv4 2>/dev/null || echo "YOUR_IP")

echo ""
echo "=== Deployment complete ==="
echo "Management UI:  http://${PUBLIC_IP}:3000"
echo "Nginx LB:       http://${PUBLIC_IP}"
echo ""
echo "Required security group rules:"
echo "  - TCP 22    (SSH)"
echo "  - TCP 80    (Nginx load balancer)"
echo "  - TCP 3000  (Management API + Web UI)"
echo "  - TCP 8001-8020 (Virtual server ports)"
