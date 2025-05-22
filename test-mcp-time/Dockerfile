FROM python:3.9-slim

# Install uv for package management
RUN pip install uv

# Install mcp-server-time package
RUN uv pip install mcp-server-time

# Set environment variables for MCP
ENV MCP_ENABLED=true
ENV MCP_STDIO=true

# Run the MCP time server
CMD ["sh", "-c", "uv pip install mcp-server-time && uvx mcp-server-time --local-timezone ${LOCAL_TIMEZONE:-UTC}"]