# ccc - Claude Code Config CLI

Quick setup tool for Claude Code configuration.

## Install

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/ducphanvanntq/ccc/main/install.ps1 | iex
```

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/ducphanvanntq/ccc/main/install.sh | bash
```

### Manual install

Download binary from [Releases](https://github.com/ducphanvanntq/ccc/releases), place it in a folder and add to PATH.

## Usage

```bash
# Init .claude config in current project (auto-applies default key)
ccc init

# Show current local config
ccc show config

# Show global default config
ccc show global

# Check API connection with current key
ccc check

# Check environment and config status
ccc doctor

# Check for updates
ccc update

# Show version
ccc version
```

### Key Management

```bash
# Interactive key manager (arrow-key menu)
ccc key

# Add a new key
ccc key add <name> <value>

# List all saved keys
ccc key list

# Set default key (saved in keys.json, used by ccc init)
ccc key default [name]

# Use a key for current folder (.claude/settings.local.json)
ccc key use [name]

# Remove a key
ccc key remove [name]

# Check if active key is valid
ccc key check
```

**default** vs **use**:
- `ccc key default` — sets which key is the global default (stored in `~/.ccc/keys.json`). Used automatically when running `ccc init`.
- `ccc key use` — applies a key to the current project folder (writes to `.claude/settings.local.json`).

---

# ccc - Claude Code Config CLI (Tiếng Việt)

Công cụ cài đặt nhanh cấu hình cho Claude Code.

## Cài đặt

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/ducphanvanntq/ccc/main/install.ps1 | iex
```

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/ducphanvanntq/ccc/main/install.sh | bash
```

### Cài thủ công

Tải binary từ [Releases](https://github.com/ducphanvanntq/ccc/releases), đặt vào một thư mục và thêm vào PATH.

## Cách dùng

```bash
# Khởi tạo cấu hình .claude trong project hiện tại (tự dùng key mặc định)
ccc init

# Xem cấu hình local hiện tại
ccc show config

# Xem cấu hình global mặc định
ccc show global

# Kiểm tra kết nối API với key hiện tại
ccc check

# Kiểm tra môi trường và trạng thái cấu hình
ccc doctor

# Kiểm tra và cập nhật phiên bản mới
ccc update

# Xem phiên bản
ccc version
```

### Quản lý Key

```bash
# Menu quản lý key (chọn bằng phím mũi tên)
ccc key

# Thêm key mới
ccc key add <tên> <giá_trị>

# Liệt kê tất cả key
ccc key list

# Đặt key mặc định (lưu trong keys.json, dùng khi ccc init)
ccc key default [tên]

# Dùng key cho folder hiện tại (.claude/settings.local.json)
ccc key use [tên]

# Xóa key
ccc key remove [tên]

# Kiểm tra key có hoạt động không
ccc key check
```

**default** vs **use**:
- `ccc key default` — đặt key mặc định toàn cục (lưu trong `~/.ccc/keys.json`). Tự động áp dụng khi chạy `ccc init`.
- `ccc key use` — áp dụng key cho project hiện tại (ghi vào `.claude/settings.local.json`).
