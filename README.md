# 🧾 Discord Expense Tracker Bot

This is a Rust-powered Discord bot that helps users track their expenses via natural language messages or uploaded bill images. It integrates with **Google Gemini** to extract expense details from images and stores them in daily logs.

---

## ✨ Features

- `$add <amount>,<category>,<note>` — Add an expense via text.
- `$total` — Get the net total of all expenses for the current day.
- `$addimage` — Upload a bill/image; Gemini will extract and log each expense line.
- Stores daily expenses in `expense/YYYY-MM-DD.txt`.

---

## 📸 Example Inputs

**Text Input**
```
$add 8000,cricket,tickets
```

**Image Upload**
```
$addimage (attach image of bill)
```

Gemini will parse the image and add lines like:
```
500,shop,snacks
1200,restaurant,dinner
```

---

## 🛠️ Setup

### 1. Clone & Navigate

```bash
git clone https://github.com/your-username/discord-expense-tracker.git
cd discord-expense-tracker
```

### 2. Set Environment Variables

Create a `.env` file:

```env
DISCORD_TOKEN=your_discord_bot_token
GEMINI_API_KEY=your_gemini_api_key
```

### 3. Install Dependencies

```bash
cargo build
```

Make sure you have Rust and `cargo` installed. Install via: https://rustup.rs

### 4. Run the Bot

```bash
cargo run
```

---

## 📁 Folder Structure

```
src/
├── main.rs         # Bot logic
├── gemini.rs       # Google Gemini API logic
expense/
└── YYYY-MM-DD.txt  # Daily logs of all expense entries
```

---

## 🧠 How It Works

- Text-based entries are directly appended to the day’s file.
- For image uploads, the image is downloaded, sent to Gemini API, and parsed into multiple entries.
- All writes are done safely using `tokio::spawn_blocking` to keep async code clean.

---

## 🚧 TODO

- Add `$summary` for category-wise totals.
- Add `$delete_last` to remove last expense.
- Allow querying by date or category.
- Store in SQLite instead of `.txt` (optional).

---

