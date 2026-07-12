// Minimal Node server for the pm2 adapter test.
const http = require("http");
const port = process.env.PORT || 3000;
http
  .createServer((_req, res) => {
    res.writeHead(200, { "Content-Type": "text/plain" });
    res.end("pm2 test site ok\n");
  })
  .listen(port, () => console.log(`pm2 test site on ${port}`));
