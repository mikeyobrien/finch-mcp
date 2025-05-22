FROM node:18-alpine

WORKDIR /app

# Copy package files first for better layer caching
COPY package*.json ./

# Install dependencies
RUN npm install --omit=dev

# Copy application code
COPY . .

# Expose the MCP server port
EXPOSE 3000

# Set non-root user for security
USER node

# Start server in appropriate mode
CMD if [ "$MCP_STDIO" = "true" ]; then \
      node index.js; \
    else \
      node index.js; \
    fi