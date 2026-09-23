# RustDesk — build do controlador (casa)

Fork de [`rustdesk/rustdesk`](https://github.com/rustdesk/rustdesk) para o projeto de acesso
remoto forense. Este build roda **só no notebook do perito** (o controlador) — nunca é o
`rustdesk.exe` do pendrive/alvo, que continua o binário vanilla 1.4.9 pinado por SHA-256.

- **Origem:** `rustdesk/rustdesk`, tag **1.4.9**, commit-base
  `6c578292e8ebbbec708b76986ba8c4bc7c509747`.
- **Branch de trabalho:** `casa/auto-hash`.
- **Objetivo do fork:** módulo `auto_hash` que, ao concluir uma cópia por File Transfer, calcula
  o SHA-256 do arquivo copiado (destino) e dispara um Terminal headless no alvo para o hash da
  origem, gravando tudo num JSONL que a `diligencia` (projeto `remote_control`) ingere na trilha.
- **Spec e plano:** em `c:\dev\remote_control` —
  `docs/superpowers/specs/2026-09-21-auto-hash-copia-rustdesk-design.md` e
  `docs/superpowers/plans/2026-09-21-auto-hash-copia-rustdesk.md`.

## Baseline

`cargo check --lib` compila limpo (só avisos) sobre o commit-base, sem patch nenhum — verificado
em 22/09/2026.

## Toolchain — versões que ESTA árvore exige (não são as últimas)

O RustDesk 1.4.9 é sensível à versão do toolchain nativo. Usar "o mais novo" quebra o build de
formas que mentem sobre a causa. As versões abaixo são as que o CI oficial do fork fixa
(`.github/workflows/flutter-build.yml`) e as únicas verificadas aqui:

| Peça | Versão exigida | Se usar outra |
|---|---|---|
| **LLVM/Clang** (`LIBCLANG_PATH`) | **15.0.6** | libclang 23 + bindgen 0.65 gera o struct `aom_codec_enc_cfg` **opaco** (só campo `_address`); `scrap` falha com 64 erros `E0609/E0560`. clang compila os headers sozinho sem erro — o defeito é só no par bindgen×libclang. |
| **vcpkg — port do aom** | **aom 3.11.0** | aom 3.15 (vcpkg HEAD) muda o layout que o `scrap` espera. RustDesk pina o vcpkg no commit `120deac306`; aqui o aom 3.11 é sobreposto sobre um vcpkg HEAD (ver abaixo). |
| **nasm** | **2.16.03** | nasm 3.x faz o configure do aom 3.11 abortar (`Unsupported nasm: multipass optimization not supported`). |
| **Visual Studio** | qualquer com workload **C++ (VC.Tools)** | O vcpkg pinado (2025) não reconhece o VS Community 2026 (v18); por isso usa-se o vcpkg HEAD, que entende o v18. |

### Por que vcpkg HEAD + overlay, e não o commit pinado do RustDesk

O RustDesk manda usar o vcpkg no commit `120deac306` (que traz aom 3.11 + nasm 2.16 casados).
Mas esse vcpkg é de 2025 e **não localiza uma instância completa do Visual Studio 2026 (v18)** —
e o VS 2022 desta máquina não tem o workload C++. O vcpkg HEAD entende o VS 2026. Solução: vcpkg
no HEAD, com dois arquivos sobrepostos do commit pinado, para casar aom↔nasm sem perder o suporte
ao VS novo:

```
cd C:\dev\vcpkg
git checkout <HEAD>                                        # tool que entende VS 2026
.\bootstrap-vcpkg.bat
git checkout 120deac306 -- ports/aom                        # aom 3.11.0
git checkout 120deac306 -- "scripts/cmake/vcpkg_find_acquire_program(NASM).cmake"   # nasm 2.16.03
```

## Build

```powershell
# dependências nativas (uma vez; compila codecs em C, ~40 min)
$env:VCPKG_ROOT = "C:\dev\vcpkg"
C:\dev\vcpkg\vcpkg.exe install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static aom:x64-windows-static

# checagem do baseline
$env:VCPKG_ROOT = "C:\dev\vcpkg"
$env:LIBCLANG_PATH = "C:\dev\tools\llvm-15\bin"
cargo check --lib
```

O build final empacotado (Flutter + `--release`) é a Task 8 do plano — pendência manual, ver o
plano em `remote_control`.
