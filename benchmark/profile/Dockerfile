# The lab's image with perf, for benchmark/profile/profile.sh.
FROM rsh-bench
RUN apt-get update -qq \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq linux-tools-generic >/dev/null \
 && ln -sf "$(ls -d /usr/lib/linux-tools/*/perf | head -1)" /usr/local/bin/perf
