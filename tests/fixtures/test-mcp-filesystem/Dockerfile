FROM node:18-alpine

WORKDIR /app

# Install dependencies
RUN npm install -g @modelcontextprotocol/server-filesystem

# Create projects directory
RUN mkdir -p /projects && chown node:node /projects

# Set working directory to /projects
WORKDIR /projects

# Set non-root user for security
USER node

# Entry point for the MCP server
ENTRYPOINT ["npx", "@modelcontextprotocol/server-filesystem"]
CMD ["/projects"]