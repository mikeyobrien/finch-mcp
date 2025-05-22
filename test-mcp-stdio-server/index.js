const express = require('express');

// Check if we're in STDIO mode
const isStdioMode = process.env.MCP_STDIO === 'true';

if (isStdioMode) {
  console.log('Starting in STDIO mode');
  
  // Simple STDIO mode handler
  process.stdin.setEncoding('utf8');
  
  process.stdin.on('data', (data) => {
    try {
      const input = data.toString().trim();
      const response = `Received: ${input}`;
      console.log(JSON.stringify({ response }));
    } catch (error) {
      console.error('Error processing input:', error);
    }
  });
  
  // Notify that we're ready
  console.log('STDIO MCP server ready');
} else {
  // HTTP mode
  const app = express();
  const PORT = process.env.PORT || 3000;

  app.use(express.json());

  app.post('/mcp', (req, res) => {
    const input = req.body.input || '';
    res.json({ response: `Received: ${input}` });
  });

  app.listen(PORT, () => {
    console.log(`MCP server listening on port ${PORT}`);
  });
}