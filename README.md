```markdown
# BAKOME AI-BOT

AI-powered assistant with email, grants scanner, mood analysis, religious quotes, humor, and crypto donation requests. Built in Rust.

## Features

- 💬 AI chat (emotion detection, humor, religious quotes)
- 📡 Subventions scanner (Gitcoin, Drips, IssueHunt, NGI0, NSF)
- ✉️ Send emails via SMTP
- 📊 Live dashboard with stats
- 🧠 Memory (SQLite) and multi-model IA (Ollama)

## Quick start

```bash
git clone https://github.com/BAKOME-Hub/BAKOME-AI-BOT.git
cd BAKOME-AI-BOT
cargo build --release
cp .env.example .env   # set your SMTP and Redis
cargo run --release
```

Configuration

Create a .env file:

```
SMTP_RELAY=smtp.gmail.com
SMTP_USER=your_email@gmail.com
SMTP_PASS=your_app_password
FROM_EMAIL=bot@localhost
REDIS_URL=redis://127.0.0.1/
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=llama3.2
```

Support

This project was built by Bakome, a self-taught developer.
If it helps you, a small donation is welcome.

Crypto addresses:

· BTC: bc1qhtjp3qpqru4vuqd355dfcn46mqjrlpdfmngk6u0
· ETH: 0x2fD73626714d9e37EA464109F8eCeA2CA5401062
· SOL: 3CfhghA7hSNPBbd1RME5rRDm5UUeesTq9NKTcyzZdkz4
· USDT (TRC20): THkLdiKsmscJFwBPA4tpWeAn1xVw7DTKxq

Thank you 🙏

License

MIT

```
```
