FROM node:20-alpine

WORKDIR /app
COPY package.json ./
RUN corepack enable
CMD ["node", "server.js"]
