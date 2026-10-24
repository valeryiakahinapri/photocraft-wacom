import os, sys, http.server, socketserver
LOG = "/home/z/logs/serve-dist.log"
def daemonize():
    if os.fork(): sys.exit(0)
    os.setsid()
    if os.fork(): os._exit(0)
    devnull = os.open('/dev/null', os.O_RDWR)
    for fd in (0, 1, 2):
        os.dup2(devnull, fd)
daemonize()
with open(LOG, "a") as f:
    f.write("serving 8765\n")
os.chdir("/home/z/pc-build/dist/web")
class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, fmt, *args): pass
class Server(socketserver.TCPServer):
    allow_reuse_address = True
with Server(("0.0.0.0", 8765), Quiet) as httpd:
    httpd.serve_forever()
