FROM node:22-alpine

WORKDIR /app
COPY package.json ./
RUN corepack enable
COPY . .
CMD ["sh", "-lc", "pnpm dev --host 0.0.0.0"]
