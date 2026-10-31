/* Study35 draft only. Linux C/libc/UAPI; no product dependency or qdisc create.
 * Source-grounding and mandatory unrun checks: README.md / CHECKS.md.
 * This has not been compiled, unit-tested or executed.
 */
#define _GNU_SOURCE
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <linux/netlink.h>
#include <linux/pkt_sched.h>
#include <linux/rtnetlink.h>
#include <net/if.h>
#include <poll.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <time.h>
#include <unistd.h>

#define POINTS 131
#define DURATION_NS UINT64_C(30000000000)
#define ACK_WAIT_NS UINT64_C(5000000000)
_Static_assert(sizeof(struct tcmsg)==20,"unexpected tcmsg ABI");
_Static_assert(sizeof(struct tc_netem_qopt)==24,"unexpected netem qopt ABI");
_Static_assert(sizeof(struct tc_netem_rate)==16,"unexpected netem rate ABI");
struct point {uint64_t offset_ns,down_kbit,up_kbit;uint32_t delay_ms,loss_pct;};
struct record {
    uint64_t mono_before_ns,wall_before_ns,wall_after_ns,mono_after_ns;
    uint32_t seq;int error;
};
struct request {struct nlmsghdr n;struct tcmsg t;unsigned char attrs[256];};
static volatile sig_atomic_t stopped;
static void stop_signal(int sig) {(void)sig;stopped=1;}
static uint64_t now_ns(clockid_t clock) {
    struct timespec t;if(clock_gettime(clock,&t)<0){perror("clock_gettime");exit(2);}
    return (uint64_t)t.tv_sec*UINT64_C(1000000000)+(uint64_t)t.tv_nsec;
}
static struct timespec as_time(uint64_t n) {
    return (struct timespec){.tv_sec=(time_t)(n/UINT64_C(1000000000)),.tv_nsec=(long)(n%UINT64_C(1000000000))};
}
/* Exactly the legacy tc6.1 clock calibration, once before the replay. */
static int psched_factor(double *factor,uint32_t values[3]) {
    FILE *f=fopen("/proc/net/psched","r");if(!f)return -1;
    int n=fscanf(f,"%x%x%x",&values[0],&values[1],&values[2]);fclose(f);
    if(n!=3 || !values[1] || !values[2]){errno=EINVAL;return -1;}
    uint32_t numerator=values[2]==1000000000U?values[1]:values[0];
    *factor=(double)numerator/(double)values[1]*((double)values[2]/1000000.0);
    if(!(*factor>0)){errno=EINVAL;return -1;}return 0;
}
static int attr(struct request *req,uint16_t type,const void *payload,size_t size) {
    size_t at=NLMSG_ALIGN(req->n.nlmsg_len),length=RTA_LENGTH(size),end=at+RTA_ALIGN(length);
    if(end>sizeof(*req)){errno=EMSGSIZE;return -1;}
    struct rtattr a={.rta_len=(unsigned short)length,.rta_type=type};
    memset((char *)req+at,0,end-at);memcpy((char *)req+at,&a,sizeof(a));
    memcpy((char *)req+at+RTA_LENGTH(0),payload,size);req->n.nlmsg_len=(uint32_t)end;return 0;
}
static int build_request(struct request *req,unsigned ifindex,uint32_t seq,const struct point *p,int up,double factor) {
    memset(req,0,sizeof(*req));req->n.nlmsg_len=NLMSG_LENGTH(sizeof(req->t));
    req->n.nlmsg_type=RTM_NEWQDISC;req->n.nlmsg_flags=NLM_F_REQUEST|NLM_F_ACK;
    req->n.nlmsg_seq=seq;req->t.tcm_family=AF_UNSPEC;req->t.tcm_ifindex=(int)ifindex;req->t.tcm_parent=TC_H_ROOT;
    double ticks=(double)p->delay_ms*1000.0*factor;
    if(ticks<0 || ticks>UINT32_MAX){errno=ERANGE;return -1;}
    struct tc_netem_qopt opt={.latency=(uint32_t)ticks,.limit=1000,.loss=p->loss_pct==100?UINT32_MAX:0};
    uint64_t rate_bytes=(up?p->up_kbit:p->down_kbit)*UINT64_C(1000)/8;
    struct tc_netem_rate rate={.rate=rate_bytes>=UINT64_C(4294967296)?UINT32_MAX:(uint32_t)rate_bytes};
    if(attr(req,TCA_KIND,"netem",6)<0)return -1;
    size_t options_at=NLMSG_ALIGN(req->n.nlmsg_len);
    if(attr(req,TCA_OPTIONS,&opt,sizeof(opt))<0)return -1;
    /* qopt followed by netem-specific attrs inside one TCA_OPTIONS, as tc6.1. */
    if(rate_bytes>=UINT64_C(4294967296) && attr(req,TCA_NETEM_RATE64,&rate_bytes,sizeof(rate_bytes))<0)return -1;
    if(attr(req,TCA_NETEM_RATE,&rate,sizeof(rate))<0)return -1;
    ((struct rtattr *)((char *)req+options_at))->rta_len=(unsigned short)(req->n.nlmsg_len-options_at);
    return 0;
}
/* Ordinary work must observe EOF even when every scheduled deadline is overdue. */
static int check_cancel(void) {
    if(stopped){errno=ECANCELED;return -1;}
    struct pollfd input={.fd=STDIN_FILENO,.events=POLLIN};
    struct timespec zero={0};int ready;
    do {ready=ppoll(&input,1,&zero,NULL);} while(ready<0 && errno==EINTR && !stopped);
    if(stopped || ready>0){stopped=1;errno=ECANCELED;return -1;}
    return ready<0?-1:0;
}
static int ack(int fd,uint32_t port,const struct request *req,int observe_cancel) {
    if(observe_cancel && check_cancel()<0)return -1;
    struct sockaddr_nl kernel={.nl_family=AF_NETLINK};
    ssize_t sent=sendto(fd,req,req->n.nlmsg_len,0,(struct sockaddr *)&kernel,sizeof(kernel));
    if(sent<0)return -1;
    if((size_t)sent!=req->n.nlmsg_len){errno=EIO;return -1;}
    uint64_t deadline=now_ns(CLOCK_MONOTONIC)+ACK_WAIT_NS;
    for(;;) {
        if(observe_cancel && check_cancel()<0)return -1;
        uint64_t now=now_ns(CLOCK_MONOTONIC);if(now>=deadline){errno=ETIMEDOUT;return -1;}
        struct timespec left=as_time(deadline-now);
        struct pollfd fds[2]={{.fd=fd,.events=POLLIN},{.fd=STDIN_FILENO,.events=POLLIN}};
        int ready=ppoll(fds,observe_cancel?2:1,&left,NULL);
        if(ready<0){
            if(errno==EINTR){if(observe_cancel && stopped){errno=ECANCELED;return -1;}continue;}
            return -1;
        }
        if(!ready){errno=ETIMEDOUT;return -1;}
        if(observe_cancel && (stopped || fds[1].revents)){stopped=1;errno=ECANCELED;return -1;}
        if(fds[0].revents&(POLLERR|POLLNVAL|POLLHUP)){errno=EIO;return -1;}
        unsigned char buffer[8192];struct sockaddr_nl sender;
        struct iovec iov={.iov_base=buffer,.iov_len=sizeof(buffer)};
        struct msghdr msg={.msg_name=&sender,.msg_namelen=sizeof(sender),.msg_iov=&iov,.msg_iovlen=1};
        ssize_t got=recvmsg(fd,&msg,MSG_DONTWAIT);
        if(got<0){if(errno==EINTR || errno==EAGAIN)continue;return -1;}
        if(msg.msg_flags&MSG_TRUNC || msg.msg_namelen!=sizeof(sender) || sender.nl_pid!=0){errno=EPROTO;return -1;}
        int remaining=(int)got;
        for(struct nlmsghdr *h=(struct nlmsghdr *)buffer;NLMSG_OK(h,remaining);h=NLMSG_NEXT(h,remaining)) {
            if(h->nlmsg_seq!=req->n.nlmsg_seq)continue; /* stale ACK cannot complete a new request */
            if(h->nlmsg_pid!=port || h->nlmsg_type!=NLMSG_ERROR || h->nlmsg_len<NLMSG_LENGTH(sizeof(struct nlmsgerr))){errno=EPROTO;return -1;}
            struct nlmsgerr error;memcpy(&error,NLMSG_DATA(h),sizeof(error));
            if(error.msg.nlmsg_seq!=req->n.nlmsg_seq || error.msg.nlmsg_type!=RTM_NEWQDISC || error.error>0){errno=EPROTO;return -1;}
            if(error.error){errno=-error.error;return -1;}return 0;
        }
        if(remaining){errno=EPROTO;return -1;}
    }
}
static int change(int fd,uint32_t port,unsigned ifindex,uint32_t seq,const struct point *p,int up,double factor,struct record *r,int observe_cancel) {
    struct request request;r->seq=seq;
    if(build_request(&request,ifindex,seq,p,up,factor)<0){r->error=errno;return -1;}
    r->mono_before_ns=now_ns(CLOCK_MONOTONIC);r->wall_before_ns=now_ns(CLOCK_REALTIME);
    int result=ack(fd,port,&request,observe_cancel);int error=result<0?errno:0;
    r->wall_after_ns=now_ns(CLOCK_REALTIME);r->mono_after_ns=now_ns(CLOCK_MONOTONIC);r->error=error;
    if(error){errno=error;return -1;}return 0;
}
/* One job. Keep stdin open until normal completion; EOF/SIGTERM cancels and restores. */
static int wait_until(uint64_t due) {
    const uint64_t spin_ns=UINT64_C(12000000),cancel_ns=UINT64_C(1000000);
    for(;;) {
        if(check_cancel()<0)return -1;
        uint64_t now=now_ns(CLOCK_MONOTONIC);if(now>=due)return 0;
        if(due-now<=spin_ns) {
            uint64_t next_cancel=now+cancel_ns;
            for(;;) {
                if(stopped){errno=ECANCELED;return -1;}
                now=now_ns(CLOCK_MONOTONIC);
                if(now>=due)return check_cancel();
                if(now>=next_cancel) {
                    if(check_cancel()<0)return -1;
                    next_cancel=now+cancel_ns;
                }
            }
        }
        struct timespec left=as_time(due-now-spin_ns);struct pollfd input={.fd=STDIN_FILENO,.events=POLLIN};
        int ready=ppoll(&input,1,&left,NULL);
        if(ready<0){if(errno==EINTR)continue;return -1;}
        if(ready){stopped=1;errno=ECANCELED;return -1;}
    }
}
static int mtu(const char *device) {
    int fd=socket(AF_INET,SOCK_DGRAM|SOCK_CLOEXEC,0);if(fd<0)return -1;
    struct ifreq request={0};strncpy(request.ifr_name,device,IFNAMSIZ-1);
    int result=ioctl(fd,SIOCGIFMTU,&request);int error=errno;close(fd);errno=error;
    return result<0?-1:request.ifr_mtu;
}
static void record_json(const struct point *p,const struct record *r) {
    printf("{\"offset_ns\":%"PRIu64",\"delay_ms\":%u,\"loss_pct\":%u,\"down_kbit\":%"PRIu64",\"up_kbit\":%"PRIu64
      ",\"seq\":%u,\"mono_before_ns\":%"PRIu64",\"wall_before_ns\":%"PRIu64",\"wall_after_ns\":%"PRIu64",\"mono_after_ns\":%"PRIu64",\"ack_error\":%d}",
      p->offset_ns,p->delay_ms,p->loss_pct,p->down_kbit,p->up_kbit,r->seq,r->mono_before_ns,r->wall_before_ns,r->wall_after_ns,r->mono_after_ns,r->error);
}
int main(int argc,char **argv) {
    if(argc!=3 || strlen(argv[1])>=IFNAMSIZ || (strcmp(argv[2],"down") && strcmp(argv[2],"up"))) {
        fprintf(stderr,"study35 draft: native-netem DEVICE down|up; input protocol in README\n");return 2;
    }
    int up=!strcmp(argv[2],"up");unsigned ifindex=if_nametoindex(argv[1]);int mtu_before=mtu(argv[1]);
    double factor;uint32_t calibration[3];
    if(!ifindex || mtu_before<0 || psched_factor(&factor,calibration)<0){perror("initial interface/psched");return 2;}
    int fd=socket(AF_NETLINK,SOCK_RAW|SOCK_CLOEXEC,NETLINK_ROUTE);struct sockaddr_nl local={.nl_family=AF_NETLINK};
    if(fd<0 || bind(fd,(struct sockaddr *)&local,sizeof(local))<0){perror("netlink open");return 2;}
    socklen_t local_length=sizeof(local);
    if(getsockname(fd,(struct sockaddr *)&local,&local_length)<0 || local_length!=sizeof(local)){perror("netlink address");close(fd);return 2;}
    struct sigaction action={.sa_handler=stop_signal};sigemptyset(&action.sa_mask);
    if(sigaction(SIGTERM,&action,NULL)<0 || sigaction(SIGINT,&action,NULL)<0){perror("signal");close(fd);return 2;}
    setvbuf(stdin,NULL,_IONBF,0);puts("ready");fflush(stdout);
    char line[256],magic[16],extra;uint64_t epoch_ns,duration;unsigned count;
    struct point points[POINTS],restore;struct record records[POINTS]={{0}},restored={0};
    if(!fgets(line,sizeof(line),stdin) || sscanf(line,"%15s %"SCNu64" %"SCNu64" %u %c",magic,&epoch_ns,&duration,&count,&extra)!=4
       || strcmp(magic,"MOVING35") || duration!=DURATION_NS || count!=POINTS)goto invalid;
    for(unsigned i=0;i<POINTS;i++) {
        struct point *p=&points[i];
        if(!fgets(line,sizeof(line),stdin) || sscanf(line,"%"SCNu64" %u %u %"SCNu64" %"SCNu64" %c",&p->offset_ns,&p->delay_ms,&p->loss_pct,&p->down_kbit,&p->up_kbit,&extra)!=5
           || p->offset_ns>=duration || (i && p->offset_ns<=points[i-1].offset_ns) || (!i && p->offset_ns)
           || p->delay_ms>10000 || (p->loss_pct!=0 && p->loss_pct!=100) || !p->down_kbit || p->down_kbit>8000 || p->up_kbit!=p->down_kbit/4)goto invalid;
    }
    memset(&restore,0,sizeof(restore));restore.offset_ns=duration;
    if(!fgets(line,sizeof(line),stdin) || sscanf(line,"%15s %u %u %"SCNu64" %"SCNu64" %c",magic,&restore.delay_ms,&restore.loss_pct,&restore.down_kbit,&restore.up_kbit,&extra)!=5
       || strcmp(magic,"RESTORE") || restore.delay_ms!=35 || restore.loss_pct || restore.down_kbit!=8000 || restore.up_kbit!=2000)goto invalid;
    uint64_t anchor_before=now_ns(CLOCK_MONOTONIC),anchor_wall=now_ns(CLOCK_REALTIME),anchor_after=now_ns(CLOCK_MONOTONIC);
    if(epoch_ns<=anchor_wall || epoch_ns-anchor_wall>UINT64_C(60000000000))goto invalid;
    uint64_t anchor_mid=anchor_before+(anchor_after-anchor_before)/2,start=anchor_mid+(epoch_ns-anchor_wall);
    unsigned completed=0,attempted=0;int error=0;
    for(unsigned i=0;i<POINTS;i++) {
        if(wait_until(start+points[i].offset_ns)<0){error=errno;break;}
        attempted=i+1;
        if(change(fd,local.nl_pid,ifindex,i+1,&points[i],up,factor,&records[i],1)<0){error=errno;break;}
        completed++;
    }
    if(!error && wait_until(start+duration)<0)error=errno;
    /* Always attempt same-qdisc restoration after an accepted plan, including cancel/failure. */
    int restore_result=change(fd,local.nl_pid,ifindex,POINTS+1,&restore,up,factor,&restored,0);
    int mtu_after=mtu(argv[1]);int success=!stopped && !error && !restore_result && completed==POINTS && mtu_after==mtu_before;
    printf("{\"schema\":35,\"epoch_ns\":%"PRIu64",\"duration_ns\":%"PRIu64",\"ifindex\":%u,\"direction\":\"%s\",\"mtu_before\":%d,\"mtu_after\":%d,\"psched\":[%u,%u,%u],\"tick_factor\":%.17g,\"anchor_mono_before_ns\":%"PRIu64",\"anchor_wall_ns\":%"PRIu64",\"anchor_mono_after_ns\":%"PRIu64",\"completed\":%u,\"error\":%d,\"stopped\":%s,\"coverage_valid\":%s,\"timing_fidelity_accepted\":false,\"records\":[",
       epoch_ns,duration,ifindex,up?"up":"down",mtu_before,mtu_after,calibration[0],calibration[1],calibration[2],factor,anchor_before,anchor_wall,anchor_after,completed,error,stopped?"true":"false",success?"true":"false");
    for(unsigned i=0;i<attempted;i++){if(i)putchar(',');record_json(&points[i],&records[i]);}
    printf("],\"restore\":");record_json(&restore,&restored);puts("}");close(fd);return success?0:1;
invalid:
    fprintf(stderr,"invalid/late study35 plan: no shaping request issued\n");close(fd);return 2;
}
