# A containerised test VPS for Popush integration tests (§23.3).
#
# It runs sshd and provides a sample site for each service type Popush supports
# (docker, systemd via a user-ish shim, pm2, static). CI spins this up and the
# backend connects over SSH and runs real operations against it. This is the test
# that catches the bugs that matter: escaping, status parsing, and the pipeline
# end to end.
#
# It never touches the developer's real environment (Agent Rule 6): everything
# lives inside the container, with a throwaway key pair generated at build time.
FROM debian:12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        openssh-server \
        git \
        curl \
        ca-certificates \
        nginx \
        docker.io docker-compose-v2 \
        nodejs npm \
    && npm install -g pm2 \
    && rm -rf /var/lib/apt/lists/*

# A non-root deploy user, matching the documented recommended setup.
RUN useradd -m -s /bin/bash deploy \
    && mkdir -p /home/deploy/.ssh \
    && chmod 700 /home/deploy/.ssh

# Authorise the test key (mounted or copied in by the harness). The public key is
# provided at build/run time; see docker-compose.yml and the README.
COPY authorized_keys /home/deploy/.ssh/authorized_keys
RUN chown -R deploy:deploy /home/deploy/.ssh \
    && chmod 600 /home/deploy/.ssh/authorized_keys

# sshd config: key auth only, no passwords (mirrors the real posture).
RUN mkdir -p /run/sshd \
    && sed -i 's/^#\?PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config \
    && sed -i 's/^#\?PubkeyAuthentication.*/PubkeyAuthentication yes/' /etc/ssh/sshd_config

# Sample sites, one per service type, under /srv.
COPY sites/ /srv/
RUN chown -R deploy:deploy /srv

EXPOSE 22
CMD ["/usr/sbin/sshd", "-D", "-e"]
