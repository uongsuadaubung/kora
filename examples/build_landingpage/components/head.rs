//! Head component: HTML Doctype, Meta tags, SEO, Fonts, CSS link

pub fn render() -> &'static str {
    r####"<!DOCTYPE html>
<html lang="vi">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Kora — Bộ gõ tiếng Việt cho Windows</title>
  <meta name="description" content="Bộ gõ tiếng Việt hiện đại viết bằng Rust. Tự động nhận diện từ tiếng Anh, không nuốt phím kép (pass, error), không nhân đôi chữ trên trình duyệt, tiêu thụ dưới 2 MB RAM.">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
  <link rel="icon" type="image/png" href="images/icon.png">
  <link rel="shortcut icon" href="favicon.ico">
  <link rel="stylesheet" href="style.css">
</head>
<body>"####
}
