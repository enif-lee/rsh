import json, subprocess, pathlib, hashlib
manifest=json.loads(pathlib.Path("/lab/manifest.json").read_text())
assert all(hashlib.sha256((pathlib.Path("/lab")/name).read_bytes()).hexdigest()==sha for name,sha in manifest.items())
print("Frozen harness SHA256 identity verified",flush=True)
import agent
results=[]
for tool, port in (("rsh-tcp",2222),("rsh-b-tcp",2223)):
    client, ready=agent.connect(tool,"functional-pair",100,30,timeout=20)
    try:
        assert ready is not None, (tool,"remote prompt absent")
        client.send("echo PAIR-$((6*7))\r")
        assert client.wait(lambda: client.has_line("PAIR-42"),10) is not None, (tool,"remote output absent")
        connections=subprocess.check_output(["ss","-tn"],text=True)
        assert any(":"+str(port) in line and "ESTAB" in line for line in connections.splitlines()), (tool,"no TCP connection on requested port")
        results.append(dict(tool=tool,port=port,connected=True,remote_output=True,tcp_established=True))
    finally:
        client.finish()
print(json.dumps(dict(functional_only=True,performance_metrics_collected=False,udp_dropped=True,results=results)))
