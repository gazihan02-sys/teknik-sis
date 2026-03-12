# SIS Teknik Platform

Rust tabanlı, modüler servis mimarisiyle tasarlanmış full-stack yönetim platformu.

## Mimari

```
sis-teknik/
├── shared/          # Ortak modeller, DTO'lar, hata tipleri
├── backend/         # Axum REST API sunucusu
│   ├── config/      # Yapılandırma
│   ├── db/          # SQLite (ilişkisel + JSON belge deposu)
│   ├── services/    # İş mantığı servisleri
│   └── routes/      # HTTP endpoint'leri
├── frontend/        # Leptos CSR + TailwindCSS
│   ├── components/  # Yeniden kullanılabilir bileşenler
│   ├── pages/       # Sayfa bileşenleri
│   └── services/    # API istemci servisleri
└── Makefile         # Kolay komut yönetimi
```

## Teknolojiler

| Katman    | Teknoloji       |
|-----------|----------------|
| Backend   | Rust + Axum    |
| Frontend  | Leptos (WASM)  |
| Stil      | TailwindCSS    |
| DB        | SQLite - teknik.db |
| Seri.     | Serde JSON     |

## Kurulum

### Gereksinimler

- Rust (rustup)
- Node.js (npm)
- trunk (`cargo install trunk`)
- wasm32 target (`rustup target add wasm32-unknown-unknown`)

### Hızlı Başlangıç

```bash
# 1. Bağımlılıkları kur
make setup

# 2. Backend'i başlat (Terminal 1)
make backend

# 3. Frontend'i başlat (Terminal 2)
make frontend
```

Backend: `http://127.0.0.1:3000`
Frontend: `http://127.0.0.1:8080`

## API Endpoints

| Method | Endpoint              | Açıklama           |
|--------|-----------------------|---------------------|
| GET    | /api/health           | Sağlık kontrolü    |
| GET    | /api/users            | Tüm kullanıcılar   |
| POST   | /api/users            | Kullanıcı oluştur  |
| GET    | /api/users/{id}       | Kullanıcı detay    |
| PUT    | /api/users/{id}       | Kullanıcı güncelle |
| DELETE | /api/users/{id}       | Kullanıcı sil      |
| GET    | /api/products         | Tüm ürünler        |
| POST   | /api/products         | Ürün oluştur       |
| GET    | /api/products/{id}    | Ürün detay         |
| DELETE | /api/products/{id}    | Ürün sil           |
