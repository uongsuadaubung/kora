fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set_manifest_file("assets/app.manifest");
        res.set("FileDescription", "Kora - Hệ sinh thái Bàn phím & Bộ gõ thế hệ mới");
        res.set("ProductName", "Kora");
        res.set("OriginalFilename", "Kora.exe");
        res.set(
            "LegalCopyright",
            "Bản quyền (C) 2026 Mạnh Kiên. All rights reserved.",
        );
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
