use parser::TranslationUnit;
use declaration::{Declaration, SingleDeclaration, TypeQualifier, VariantTypeSpecifier};

pub trait QueryType {
    type Type;
    type ResultType;
}

pub trait Query<'a> {
    fn query_decl(&self, s: &str) -> Option<&SingleDeclaration>;

    fn query_decl_all<'b: 'a, T>(&'b self, t: T) -> Vec<&'b SingleDeclaration>
    where
        T: DeclQuery;
}

pub trait SingleDeclarationQuery<'a> {
    fn is<T>(self, t: T) -> Self
    where
        T: DeclQuery;
}

impl<'a> SingleDeclarationQuery<'a> for Option<&'a SingleDeclaration> {
    fn is<T>(self, t: T) -> Self
    where
        T: DeclQuery,
    {
        if let Some(ref sdec) = self {
            if t.is(sdec) {
                return self;
            }
        }
        None
    }
}

pub trait DeclQuery {
    fn is(&self, decl: &SingleDeclaration) -> bool;
}

impl DeclQuery for TypeQualifier {
    fn is(&self, decl: &SingleDeclaration) -> bool {
        if let VariantTypeSpecifier::Normal(ref full_ts) = decl.variant_type_spec {
            if let Some(ref tq) = full_ts.qualifer {
                if *tq == *self {
                    return true;
                }
            }
        }

        false
    }
}

impl<'a> Query<'a> for TranslationUnit {
    fn query_decl(&self, s: &str) -> Option<&SingleDeclaration> {
        for decl in self.decls.iter() {
            if let &Declaration::DeclarationList(ref list) = decl {
                for sdecl in list.iter() {
                    if let Some(ref name) = sdecl.name {
                        if name == s {
                            return Some(sdecl);
                        }
                    }
                }
            }
        }

        None
    }

    fn query_decl_all<'b: 'a, T>(&'b self, t: T) -> Vec<&'b SingleDeclaration>
    where
        T: DeclQuery,
    {
        let mut res = Vec::new();

        for decl in self.decls.iter() {
            if let &Declaration::DeclarationList(ref list) = decl {
                for sdecl in list.iter() {
                    if t.is(sdecl) {
                        res.push(sdecl);
                    }
                }
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::parse;

    #[test]
    fn preprocess_test_query_decl() {
        let test_text = r#"attribute vec3 pos;"#;
        let unit = parse(test_text).unwrap();
        let decl: Option<&SingleDeclaration> = unit.query_decl("pos").is(TypeQualifier::Attribute);

        assert!(decl.is_some());
    }

    #[test]
    fn preprocess_test_query_decl_all() {
        let test_text = r#"uniform vec3 p; uniform vec3 v;"#;
        let unit = parse(test_text).unwrap();

        let decl: Vec<&SingleDeclaration> = unit.query_decl_all(TypeQualifier::Uniform);

        assert_eq!(decl.len(), 2);
    }
}
