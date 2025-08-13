use std::{fmt::Display, str::FromStr};

pub struct AuthToken {
    token_type: TokenType,
    value: String,
}

enum TokenType {
    Basic,
    Bearer,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Basic => "Basic",
                Self::Bearer => "Bearer",
            }
        )
    }
}

impl AuthToken {
    pub fn from_basic(token: &str) -> Self {
        Self {
            token_type: TokenType::Basic,
            value: token.to_string()
        }
    }
    
    pub fn from_bearer(token: &str) -> Self {
        Self {
            token_type: TokenType::Bearer,
            value: token.to_string()
        }
    }
    
    pub fn get_type(&self) -> &'static str {
        match &self.token_type {
            TokenType::Basic => "Basic",
            TokenType::Bearer => "Bearer"
        }
    }
    
    pub fn get_value(&self) -> &str {
        &self.value
    }
}