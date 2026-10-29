# One image for both ends of the comparison: OpenSSH, mosh, Eternal
# Terminal (PPA), rsh (this repo's Linux build), and the measuring scripts.
FROM ubuntu:24.04
RUN apt-get update \
 && apt-get install -y --no-install-recommends software-properties-common gpg-agent \
 && add-apt-repository -y ppa:jgmath2000/et \
 && apt-get update \
 && apt-get install -y --no-install-recommends openssh-server openssh-client mosh et \
      python3 python3-pyte iproute2 iptables procps \
 && rm -rf /var/lib/apt/lists/*
COPY rsh /usr/local/bin/rsh
COPY anim.py oscs.py sample.py drive.py start-server.sh /usr/local/bin/
RUN useradd -m -s /bin/bash dev && mkdir -p /run/sshd \
 && ssh-keygen -q -t ed25519 -N '' -f /root/.ssh/id_ed25519 \
 && mkdir -p /home/dev/.ssh && cp /root/.ssh/id_ed25519.pub /home/dev/.ssh/authorized_keys \
 && printf 'Host *\n  StrictHostKeyChecking no\n  UserKnownHostsFile /dev/null\n  LogLevel ERROR\n' > /root/.ssh/config \
 && chown -R dev:dev /home/dev/.ssh && chmod 700 /home/dev/.ssh && chmod 600 /home/dev/.ssh/authorized_keys \
 && mkdir -p /home/dev/.rsh /root/.rsh \
 && echo "client:bench benchtoken0123456789" > /home/dev/.rsh/authorized \
 && echo "server benchtoken0123456789" > /root/.rsh/known \
 && chown -R dev:dev /home/dev/.rsh && chmod 700 /home/dev/.rsh /root/.rsh \
 && chmod 600 /home/dev/.rsh/authorized /root/.rsh/known \
 && chmod +x /usr/local/bin/*.py /usr/local/bin/start-server.sh
ENV LANG=C.UTF-8
CMD ["sleep", "infinity"]
