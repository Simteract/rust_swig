use std::fmt::{Display, Write};

use crate::{
    source_registry::{SourceId, SourceRegistry},
    SourceCode, WRITE_TO_MEM_FAILED_MSG,
};
use proc_macro2::Span;

pub(crate) type SourceIdSpan = (SourceId, Span);

pub(crate) fn invalid_src_id_span() -> SourceIdSpan {
    (SourceId::none(), Span::call_site())
}

#[derive(Debug)]
pub(crate) struct DiagnosticError {
    data: Vec<(SourceId, syn::Error)>,
}

impl DiagnosticError {
    pub fn from_syn_err(src_id: SourceId, err: syn::Error) -> Self {
        DiagnosticError {
            data: vec![(src_id, err)],
        }
    }
    pub fn new<T: Display>(src_id: SourceId, sp: Span, err: T) -> Self {
        DiagnosticError {
            data: vec![(src_id, syn::Error::new(sp, err))],
        }
    }
    pub fn new2<T: Display>((src_id, sp): SourceIdSpan, err: T) -> Self {
        DiagnosticError {
            data: vec![(src_id, syn::Error::new(sp, err))],
        }
    }
    pub fn span_note<T: Display>(&mut self, sp: SourceIdSpan, err: T) {
        self.data.push((sp.0, syn::Error::new(sp.1, err)));
    }
    pub fn add_span_note<T: Display>(mut self, sp: SourceIdSpan, err: T) -> Self {
        self.span_note(sp, err);
        self
    }
    pub fn new_without_src_info<T: Display>(err: T) -> Self {
        DiagnosticError {
            data: vec![(SourceId::none(), syn::Error::new(Span::call_site(), err))],
        }
    }
    pub(crate) fn map_any_err_to_our_err<E: Display>(err: E) -> Self {
        DiagnosticError::new_without_src_info(err)
    }
}

impl Display for DiagnosticError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        for x in &self.data {
            write!(f, "{}", x.1)?;
        }
        Ok(())
    }
}

impl std::error::Error for DiagnosticError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.data.first()?.1)
    }
}

impl From<std::io::Error> for DiagnosticError {
    fn from(err: std::io::Error) -> Self {
        DiagnosticError::new_without_src_info(err)
    }
}

impl From<syn::Error> for DiagnosticError {
    fn from(err: syn::Error) -> Self {
        DiagnosticError::new_without_src_info(err)
    }
}

pub(crate) trait ResultDiagnostic<T> {
    fn with_span(self, sp: SourceIdSpan) -> std::result::Result<T, DiagnosticError>;
    fn with_span_note<D: Display>(self, sp: SourceIdSpan, d: D) -> std::result::Result<T, DiagnosticError>;
    fn with_note<D: Display>(self, d: D) -> std::result::Result<T, DiagnosticError>;
}

impl<T, E: Display> ResultDiagnostic<T> for std::result::Result<T, E> {
    fn with_span(self, sp: SourceIdSpan) -> std::result::Result<T, DiagnosticError> {
        self.map_err(|err| DiagnosticError::new2(sp, err))
    }
    
    fn with_span_note<D: Display>(self, sp: SourceIdSpan, d: D) -> std::result::Result<T, DiagnosticError> {
        self.map_err(|err| DiagnosticError::new2(sp, err).add_span_note(sp, d))
    }
    
    fn with_note<D: Display>(self, d: D) -> std::result::Result<T, DiagnosticError> {
        self.map_err(|err| DiagnosticError::new_without_src_info(err).add_span_note(invalid_src_id_span(), d))
    }
    
}

pub(crate) trait ResultSynDiagnostic<T>: ResultDiagnostic<T> {
    fn with_syn_src_id(self, src_id: SourceId) -> std::result::Result<T, DiagnosticError>;
}

impl <T> ResultSynDiagnostic<T> for std::result::Result<T, syn::Error> {
    fn with_syn_src_id(self, src_id: SourceId) -> std::result::Result<T, DiagnosticError> {
        self.map_err(|err| DiagnosticError::from_syn_err(src_id, err))
    }
}

pub(crate) type Result<T> = std::result::Result<T, DiagnosticError>;

pub(crate) fn panic_on_syn_error(id_of_code: &str, code: String, err: syn::Error) -> ! {
    let mut src_reg = SourceRegistry::default();
    let src_id = src_reg.register(SourceCode {
        id_of_code: id_of_code.into(),
        code,
    });
    panic_on_parse_error(&src_reg, &DiagnosticError::from_syn_err(src_id, err));
}

pub(crate) fn panic_on_parse_error(src_reg: &SourceRegistry, main_err: &DiagnosticError) -> ! {
    let mut prev_err_src_id = None;

    for (src_id, err) in &main_err.data {
        if src_id.is_none() {
            eprintln!("Error (without location information): {}", err);
            continue;
        }
        let src = &src_reg.src_with_id(*src_id);
        if prev_err_src_id.map(|id| id != *src_id).unwrap_or(true) {
            eprintln!("error in {}", src.id_of_code);
        }
        prev_err_src_id = Some(*src_id);
        eprint_error_location(err, src);
    }
    panic!();
}

fn eprint_error_location(err: &syn::Error, src: &SourceCode) {
    let span = err.span();
    let start = span.start();
    let end = span.end();

    let mut code_problem = String::new();
    let nlines = end.line - start.line + 1;
    for (i, line) in src
        .code
        .lines()
        .skip(start.line - 1)
        .take(nlines)
        .enumerate()
    {
        code_problem.push_str(&line);
        code_problem.push('\n');
        if i == 0 && start.column > 0 {
            write!(&mut code_problem, "{:1$}", ' ', start.column).expect("write to String failed");
        }
        let code_problem_len = if i == 0 {
            if i == nlines - 1 {
                end.column - start.column
            } else {
                line.len() - start.column - 1
            }
        } else if i != nlines - 1 {
            line.len()
        } else {
            end.column
        };
        writeln!(&mut code_problem, "{:^^1$}", '^', code_problem_len)
            .expect(WRITE_TO_MEM_FAILED_MSG);
        if i == end.line {
            break;
        }
    }

    eprintln!(
        "parsing of {name} failed\nerror: {err}\n{code_problem}\nAt {name}:{line_s}:{col_s}",
        name = src.id_of_code,
        err = err,
        code_problem = code_problem,
        line_s = start.line,
        col_s = start.column,
    );
}
