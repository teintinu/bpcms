//! Posições no texto: offsets de byte e conversão para linha/coluna.

use serde::Serialize;

/// Intervalo `[start, end)` em bytes do arquivo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn at(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}

/// Índice de linhas para converter offset em `(linha, coluna)`, ambos base 0.
pub struct LineIndex {
    newlines: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let newlines = source
            .bytes()
            .enumerate()
            .filter_map(|(i, b)| (b == b'\n').then_some(i))
            .collect();
        Self { newlines }
    }

    pub fn line_col(&self, offset: usize) -> (u32, u32) {
        let line = self.newlines.partition_point(|&nl| nl < offset);
        let line_start = if line == 0 {
            0
        } else {
            self.newlines[line - 1] + 1
        };
        (line as u32, (offset.saturating_sub(line_start)) as u32)
    }

    pub fn line_start(&self, line: u32) -> usize {
        if line == 0 {
            0
        } else {
            self.newlines
                .get(line as usize - 1)
                .map(|nl| nl + 1)
                .unwrap_or(usize::MAX)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converte_offsets_em_linha_e_coluna() {
        let index = LineIndex::new("abc\nde\nf");
        assert_eq!(index.line_col(0), (0, 0));
        assert_eq!(index.line_col(2), (0, 2));
        assert_eq!(index.line_col(4), (1, 0));
        assert_eq!(index.line_col(7), (2, 0));
        assert_eq!(index.line_start(1), 4);
    }
}
