// Módulo auto_hash: schema JSONL, escrita append, e MarkerScanner para
// detectar o fim de saída de um comando executado em terminal headless.
//
// Task 3 do plano `2026-09-21-auto-hash-copia-rustdesk`: base pura, sem
// rede — não registrado em `lib.rs` (isso é Task 4).

use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
pub struct AcaoCopiaArquivo {
    pub ts_utc: String,
    pub mecanismo: String,
    pub caminho_remoto: String,
    pub caminho_local: String,
    pub tamanho: u64,
    pub sha256_local: String,
    pub saida_hash_remoto_bruta: Option<String>,
    pub erro_hash_origem: Option<String>,
}

pub fn caminho_log(pasta_gravacao: &Path, prefixo_video: &str) -> PathBuf {
    pasta_gravacao.join(format!("{prefixo_video}_acoes.jsonl"))
}

pub fn registrar_linha(caminho_log: &Path, acao: &AcaoCopiaArquivo) -> std::io::Result<()> {
    let linha = serde_json::to_string(acao)?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(caminho_log)?;
    writeln!(f, "{linha}")?;
    f.sync_all()
}

pub struct MarkerScanner {
    marcador: Vec<u8>,
    buffer: Vec<u8>,
}

impl MarkerScanner {
    pub fn new(marcador: &str) -> Self {
        Self {
            marcador: marcador.as_bytes().to_vec(),
            buffer: Vec::new(),
        }
    }

    /// `Some(saida_antes_do_marcador)` na primeira vez que o marcador aparece
    /// (acumulado desde o último reset interno); `None` enquanto não achou —
    /// o restante fica retido no buffer pra próxima chamada.
    ///
    /// Nota (achado por TDD, Task 3): a versão original do brief mantinha só
    /// a cauda de `marcador.len() - 1` bytes a cada chamada sem match, pra
    /// não crescer sem limite numa saída longa sem marcador. Isso descarta
    /// bytes de saída legítima que precisam ser devolvidos como `saida`
    /// quando o marcador aparece fragmentado numa chamada seguinte — o teste
    /// `marcador_fragmentado_entre_leituras` pegou isso (RED real, não só
    /// falta de símbolo). O buffer não é mais truncado: cresce até o
    /// marcador aparecer e então é limpo por inteiro. Para os usos previstos
    /// (saída curta de um comando com marcador conhecido num terminal
    /// headless) isso não é um problema prático de memória; se algum dia for
    /// preciso um teto, o teto tem que preservar o prefixo acumulado, não
    /// descartá-lo.
    pub fn push(&mut self, chunk: &[u8]) -> Option<Vec<u8>> {
        // Marcador vazio nunca casa; guarda contra o panic de `windows(0)`.
        if self.marcador.is_empty() {
            return None;
        }
        self.buffer.extend_from_slice(chunk);
        if let Some(pos) = self
            .buffer
            .windows(self.marcador.len())
            .position(|w| w == self.marcador.as_slice())
        {
            let saida = self.buffer[..pos].to_vec();
            self.buffer.clear();
            return Some(saida);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marcador_numa_unica_leitura() {
        let mut s = MarkerScanner::new("###FIM###");
        let r = s.push(b"Hash: abc\r\n###FIM###resto");
        assert_eq!(r, Some(b"Hash: abc\r\n".to_vec()));
    }

    #[test]
    fn marcador_fragmentado_entre_leituras() {
        let mut s = MarkerScanner::new("###FIM###");
        assert_eq!(s.push(b"Hash: abc\r\n##"), None);
        assert_eq!(s.push(b"#FIM"), None);
        assert_eq!(s.push(b"###resto"), Some(b"Hash: abc\r\n".to_vec()));
    }

    #[test]
    fn sem_marcador_continua_none() {
        let mut s = MarkerScanner::new("###FIM###");
        assert_eq!(s.push(b"ainda saindo dado"), None);
    }

    #[test]
    fn caminho_log_usa_sufixo_acoes_jsonl() {
        // Trava o sufixo `_acoes.jsonl` — o coletor Python (diligencia) casa
        // exatamente esse glob; se o nome mudar, a ingestão silenciosamente
        // não acha nada.
        let dir = Path::new(r"C:\gravacoes");
        assert_eq!(
            caminho_log(dir, "sessao_2026"),
            dir.join("sessao_2026_acoes.jsonl")
        );
    }

    #[test]
    fn marcador_vazio_nunca_casa_sem_panic() {
        let mut s = MarkerScanner::new("");
        assert_eq!(s.push(b"qualquer coisa"), None);
    }

    #[test]
    fn registrar_linha_grava_jsonl_valido() {
        let tmp = tempfile::tempdir().unwrap();
        let acao = AcaoCopiaArquivo {
            ts_utc: "2026-09-21T17:25:13.175Z".into(),
            mecanismo: "file_transfer".into(),
            caminho_remoto: r"C:\Users\alvo\Documents\a.pdf".into(),
            caminho_local: r"C:\Users\perito\Downloads\a.pdf".into(),
            tamanho: 100,
            sha256_local: "a".repeat(64),
            saida_hash_remoto_bruta: None,
            erro_hash_origem: Some("timeout".into()),
        };
        let log = tmp.path().join("sessao_acoes.jsonl");
        registrar_linha(&log, &acao).unwrap();
        let conteudo = std::fs::read_to_string(&log).unwrap();
        let linha: serde_json::Value = serde_json::from_str(conteudo.trim()).unwrap();
        assert_eq!(linha["mecanismo"], "file_transfer");
        assert_eq!(linha["erro_hash_origem"], "timeout");
    }
}
