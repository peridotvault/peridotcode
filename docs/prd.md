# Product Requirements Document (PRD)

## Product Name

**PeridotCode**

## Tagline

**Build games with prompts. Ship them with Peridot.**

---

## 1. Executive Summary

PeridotCode adalah **terminal-first AI game creation agent** yang membantu developer membuat **playable game prototype** dari prompt. Produk ini bukan game engine baru, melainkan **agentic development layer** yang berjalan di atas stack game yang sudah ada, dimulai dari template-driven scaffolding untuk game 2D sederhana.

PeridotCode dirancang sebagai bagian strategis dari ekosistem Peridot. Dalam jangka panjang, PeridotCode tidak hanya membantu developer **membuat game lebih cepat**, tetapi juga membantu mereka **menyiapkan game untuk integrasi dan distribusi** ke PeridotVault.

Fokus fase awal:

- menerima prompt dari user,
- memahami intent,
- memilih template,
- menghasilkan project runnable yang bisa diedit,
- dan menggunakan model AI yang dipilih user melalui provider seperti OpenRouter.

---

## 2. Background and Context

PeridotVault saat ini bergerak di sisi **distribution layer** untuk game. Namun distribution saja belum cukup kuat sebagai pembeda jangka panjang. Salah satu peluang strategis terbesar adalah masuk lebih awal ke **creation layer**.

Di sisi developer, banyak masalah yang terus berulang:

- prototyping lambat,
- setup project berulang,
- boilerplate memakan waktu,
- tool AI umum belum cukup game-native,
- dan creation workflow tidak terhubung ke distribution workflow.

PeridotCode hadir untuk mengisi gap ini.

PeridotCode memperluas posisi Peridot dari:
**distribution platform**
menjadi:
**creation + integration + distribution ecosystem**

---

## 3. Problem Statement

### Core Problem

Membuat prototype game yang playable masih terlalu lambat, terlalu manual, dan terlalu banyak boilerplate.

### Supporting Problems

#### 3.1 Game prototyping is slow

Developer harus membuat folder structure, bootstrap project, scene/script awal, dan sistem dasar sebelum ada hasil yang bisa dimainkan.

#### 3.2 Repeated boilerplate slows momentum

Movement, UI, menu, dialogue, inventory, save/load, dan loop dasar sering harus diulang dari nol.

#### 3.3 Existing AI coding tools are not game-native enough

AI coding tools umum bisa membantu generate code, tetapi belum benar-benar memahami workflow game development secara terarah.

#### 3.4 Development and distribution are disconnected

Setelah prototype jadi, developer masih harus melakukan langkah terpisah untuk integrasi, packaging, metadata, dan publishing.

#### 3.5 Model/provider lock-in is undesirable

Developer tooling AI sering terlalu terikat ke satu model atau provider. PeridotCode perlu fleksibel agar user bisa memilih model sendiri sesuai biaya, preferensi, dan performa.

---

## 4. Product Vision

PeridotCode menjadi AI-native game development agent yang memungkinkan developer pergi dari **idea → playable prototype** dalam waktu singkat, langsung dari terminal, dengan model yang bisa dipilih sendiri.

### Vision Statement

**Enable developers to go from idea to playable game prototype in minutes through a terminal-first AI workflow, then eventually prepare those games to ship into PeridotVault.**

---

## 5. Product Positioning

### What PeridotCode is

- terminal-first AI game creation agent
- developer tool untuk prompt-to-game prototyping
- agentic layer di atas framework/engine yang sudah ada
- model-agnostic AI tooling
- future on-ramp ke ekosistem PeridotVault

### What PeridotCode is not

- bukan game engine baru
- bukan full no-code game builder
- bukan replacement untuk Unity, Godot, atau Unreal
- bukan sekadar chatbot coding biasa
- bukan publishing platform itu sendiri

### Positioning Statement

**PeridotCode is a terminal-first AI game creation agent that turns prompts into playable prototypes using the model provider chosen by the developer.**

---

## 6. Goals

### Primary Goals

1. Memungkinkan developer membuat playable game prototype dari prompt.
2. Mengurangi waktu setup project dan boilerplate.
3. Menyediakan terminal-native UX yang terasa cepat dan profesional.
4. Mendukung multi-provider model selection melalui user-supplied API keys.
5. Menjadi creation-layer wedge yang memperkuat posisi PeridotVault.

### Secondary Goals

1. Menjadi diferensiasi strategis Peridot dibanding distribution platform lain.
2. Menarik indie developers dan builder technical ke ekosistem Peridot.
3. Menyediakan fondasi untuk skill modular seperti inventory, dialogue, save system, dan nantinya Peridot integration.
4. Menghindari ketergantungan pada satu vendor model.

---

## 7. Non-Goals

Untuk fase awal, PeridotCode tidak akan fokus pada:

- membuat AAA games,
- menggantikan full workflow engine profesional,
- mendukung semua engine sekaligus,
- full visual editor,
- publishing langsung ke PeridotVault,
- advanced autonomous multi-step agent loop,
- multiplayer kompleks,
- billing internal AI,
- automatic cost optimization across providers,
- OS keychain integration yang sangat kompleks,
- plugin marketplace penuh.

---

## 8. Target Users

### Primary Users

#### 8.1 Indie Game Developers

- solo dev atau tim kecil
- ingin prototyping lebih cepat
- nyaman dengan tooling developer

#### 8.2 Technical Builders / Hackers

- suka CLI/TUI workflow
- ingin hasil cepat dan editable
- terbuka pada AI-assisted generation

#### 8.3 Early Peridot Ecosystem Developers

- calon developer yang nanti akan publish atau integrate ke PeridotVault
- membutuhkan jalur tercepat dari ide ke prototype

### Secondary Users

#### 8.4 Beginner Developers

- masih belajar game dev
- ingin scaffold yang bisa dipelajari

#### 8.5 Hackathon Builders

- perlu demo atau prototype dalam waktu singkat

---

## 9. User Personas

### Persona A — Solo Indie Developer

Sudah pernah memakai game framework atau engine, tetapi sering kehilangan momentum di tahap awal setup dan sistem dasar.

### Persona B — Terminal-First Builder

Lebih nyaman bekerja dari CLI/TUI, ingin AI yang terasa seperti developer tool sungguhan, bukan sekadar chat wrapper.

### Persona C — Experimental Creator

Punya banyak ide dan ingin mengujinya dengan cepat tanpa membangun semuanya dari nol.

---

## 10. User Problems to Solve

1. “Aku punya ide game, tapi malas atau lambat mulai dari nol.”
2. “Aku butuh prototype playable secepat mungkin.”
3. “Aku capek bikin struktur, movement, menu, dan sistem dasar berulang-ulang.”
4. “Aku ingin bisa memilih model AI sendiri.”
5. “Aku tidak mau tool ini terkunci ke satu provider.”
6. “Aku ingin hasil yang tetap bisa kubaca dan edit sendiri.”

---

## 11. Core Value Proposition

### Functional Value

- mengubah prompt menjadi project scaffold runnable
- mengurangi setup manual
- mempercepat prototyping
- memberikan hasil yang editable
- memungkinkan user memilih provider dan model sendiri

### Strategic Value

- menambah creation layer ke Peridot ecosystem
- membuka jalur dari build ke ship
- memberi developer-side moat untuk Peridot

### Emotional Value

- mengurangi blank page anxiety
- membuat ide terasa cepat dieksekusi
- memberi rasa “power tool” bagi developer

---

## 12. Product Scope Overview

### MVP Scope

- Rust-based terminal-first CLI/TUI
- prompt input
- orchestration dasar
- template-driven scaffold generation
- 1 template awal: Phaser 2D starter
- file generation summary
- basic project context awareness
- provider/model abstraction
- OpenRouter support first
- user-supplied API key configuration
- default provider/model selection

### Future Scope

- OpenAI direct support
- Anthropic direct support
- Gemini support
- local models
- skills modular
- add-feature flow
- Peridot auth skill
- publishing preparation
- analytics and achievements skills

---

## 13. MVP Definition

### MVP Thesis

MVP bukan “AI yang bisa membuat game apa saja”.

MVP adalah:

**terminal-first Rust tool yang menerima prompt, menggunakan model pilihan user, lalu menghasilkan playable Phaser 2D starter project yang editable dan runnable.**

### MVP Success Shape

User:

1. menjalankan `peridotcode`
2. setup provider jika belum ada
3. memilih model
4. memasukkan prompt
5. sistem memahami intent
6. sistem memilih template
7. sistem menghasilkan project scaffold
8. user mendapat file summary dan run instructions

---

## 14. Product Principles

1. **Terminal-first**
   Produk harus terasa seperti tool developer sungguhan.

2. **Constrained generation over magical chaos**
   Lebih baik hasil stabil dan runnable daripada liar tapi tidak bisa dipakai.

3. **Playable first**
   Fokus awal adalah output yang bisa dijalankan.

4. **Human-editable output**
   Semua hasil harus tetap bisa dibaca dan diubah developer.

5. **Model-agnostic**
   User harus bisa memilih provider dan model.

6. **OpenRouter-first, not OpenRouter-only**
   OpenRouter jadi prioritas awal, tapi arsitektur tidak boleh terkunci.

7. **Composable future**
   Sistem harus siap untuk skills modular.

---

## 15. Key Use Cases

### 15.1 Create a New Game Prototype

User membuka folder kosong, menjalankan `peridotcode`, memberikan prompt, lalu mendapat project starter yang playable.

### 15.2 Configure Provider and Model

User memilih provider, memasukkan API key, memilih model default, dan menyimpan konfigurasi.

### 15.3 Iterate on Prompt-Based Generation

User memperbaiki prompt, meminta scaffold baru, atau meminta penyesuaian sederhana.

### 15.4 Future Add-Skill Flow

Di fase berikutnya, user bisa menambahkan skill seperti dialogue, inventory, atau save system.

---

## 16. User Journey

### Journey A — First-Time Setup

1. user menjalankan `peridotcode`
2. sistem mendeteksi belum ada provider configured
3. sistem menampilkan setup flow
4. user memilih provider
5. user memasukkan API key atau env reference
6. sistem memvalidasi
7. user memilih default model
8. setup selesai

### Journey B — New Project Creation

1. user masuk ke folder project
2. user menjalankan `peridotcode`
3. TUI terbuka
4. user memasukkan prompt
5. orchestrator memproses prompt
6. model gateway dipakai jika perlu
7. template dipilih
8. file dihasilkan
9. user melihat summary dan run instructions

---

## 17. Functional Requirements

### 17.1 CLI / TUI

- CLI entrypoint `peridotcode`
- terminal UI shell
- current working directory awareness
- prompt input field
- task/status panel
- file summary panel

### 17.2 Project Context

- deteksi folder kosong vs existing project
- baca struktur dasar project
- berikan context minimum ke orchestrator

### 17.3 Orchestration

- menerima prompt
- mengklasifikasikan intent sederhana
- menghasilkan execution plan
- memanggil template engine
- menampilkan hasil ke UI

### 17.4 Template Engine

- template registry
- template selection
- template rendering/scaffolding
- file output summary
- validasi dasar

### 17.5 File System Engine

- safe read/write
- prevent writing outside project scope
- summary perubahan file

### 17.6 Command Runner

- doctor/diagnostic
- run instructions
- safe process execution abstraction

### 17.7 Model Gateway

- provider abstraction
- model abstraction
- configuration loading
- credential reference support
- default provider/model support
- normalized inference request/response

### 17.8 Provider Support

- OpenRouter first
- OpenAI / Anthropic / Gemini later
- future local model support

### 17.9 Configuration

- config file support
- `.env` support
- provider enable/disable
- default provider/model selection

---

## 18. Non-Functional Requirements

### Performance

- startup cepat
- setup flow terasa ringan
- scaffold generation memberi feedback progres

### Reliability

- tidak mudah merusak file user
- error state jelas
- config parsing tidak rapuh

### Usability

- UX terminal tetap jelas
- setup provider sederhana
- command output singkat tapi informatif

### Extensibility

- mudah tambah provider baru
- mudah tambah model metadata
- mudah tambah skill system nanti

### Maintainability

- Rust workspace modular
- crate boundaries jelas
- tanggung jawab antar modul tidak tumpang tindih

### Security

- jangan hardcode API keys
- gunakan env reference atau credential abstraction
- future-ready untuk secure storage

---

## 19. Recommended Technical Direction

### Core Stack

- Rust
- Cargo workspace
- terminal-first CLI/TUI
- template-driven generation
- HTTP/API-based model provider integration

### Recommended High-Level Components

- CLI crate
- TUI crate
- Core orchestrator crate
- Template engine crate
- FS engine crate
- Command runner crate
- Skills crate
- Shared crate
- Model gateway crate

---

## 20. Template Strategy

### MVP Template

**Phaser 2D starter**

### Why

- cepat didemo
- ringan
- mudah dirender sebagai runnable prototype
- cocok untuk MVP prompt-to-playable

### Principle

Templates harus deterministic, kecil, mudah dipahami, dan mudah di-edit.

---

## 21. Model Provider Strategy

### Initial Strategy

- OpenRouter as priority provider
- user-supplied API key
- default model selection
- recommended model metadata

### Future Strategy

- OpenAI direct
- Anthropic direct
- Gemini
- local model support

### Design Principle

PeridotCode harus **model-agnostic**, bukan provider-locked.

---

## 22. Success Metrics

### Product Metrics

- time to first playable prototype
- setup completion rate
- successful scaffold generation rate
- second-session retention
- number of generated projects per user

### Model/Provider Metrics

- provider setup success rate
- model selection completion rate
- inference success rate
- frequency of provider use by type

### Strategic Metrics

- number of developers entering Peridot ecosystem through PeridotCode
- future conversion from creation tool to Peridot platform usage

---

## 23. MVP Success Criteria

MVP dianggap berhasil jika:

- user bisa menjalankan `peridotcode`
- user bisa setup provider/model
- user bisa memasukkan prompt
- sistem bisa menghasilkan playable Phaser starter project
- file summary dan run instructions tampil jelas
- codebase cukup modular untuk menerima skill system dan provider tambahan

---

## 24. Risks and Mitigations

### Risk 1 — Scope explosion

Mitigasi: satu stack, satu template, satu happy path dulu.

### Risk 2 — Overengineering provider layer

Mitigasi: buat abstraction cukup kecil, jangan terlalu generik.

### Risk 3 — Poor output quality

Mitigasi: template-first, constrained generation, validation sederhana.

### Risk 4 — User confusion around models

Mitigasi: recommended models, supported labels, simple setup flow.

### Risk 5 — Product loses focus

Mitigasi: tetap anggap ini prototype generator, bukan full AI engine replacement.

---

## 25. Release Strategy

### Phase 0 — Internal Scaffold

repo, docs, crate boundaries, CLI/TUI shell

### Phase 1 — MVP Foundation

prompt intake, orchestrator, template engine, file safety, OpenRouter support

### Phase 2 — Better UX

provider commands, model catalog, improved setup flow

### Phase 3 — Multi-Provider

OpenAI, Anthropic, Gemini

### Phase 4 — Skills & Peridot Future

inventory/dialogue/save-system skills, later Peridot integration

---

## 26. Strategic Role in Peridot Ecosystem

PeridotCode memberi Peridot sesuatu yang lebih besar dari sekadar distribusi.

### Strategic Narrative

**Peridot is not only where games are distributed. It is where games begin.**

PeridotCode adalah creation layer yang nanti bisa menjadi pintu masuk developer ke seluruh ekosistem Peridot.

---

## 27. Final Product Statement

**PeridotCode is a Rust-based terminal-first AI game creation agent that helps developers turn prompts into playable prototypes using the model provider of their choice.**
