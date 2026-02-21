# Use the lightweight Alpine Linux version of Node to drastically reduce image size
FROM node:20-alpine

# Set the working directory inside the container
WORKDIR /app

# Set environment to production to ensure frameworks optimize their execution
ENV NODE_ENV=production

# Copy only the dependency definitions first to leverage Docker layer caching.
# If package.json doesn't change, Docker skips running npm install on rebuilds.
COPY package*.json ./

# Install dependencies.
# Using a clean install if a lockfile exists, otherwise fallback to install.
RUN npm ci || npm install

# Copy the rest of the application code
COPY . .

# SECURITY: Run the container as the unprivileged 'node' user instead of 'root'
# This prevents privilege escalation attacks if the bot is compromised.
USER node

# Command to start the Keeper bot as specified in the README
CMD ["node", "index.js"]