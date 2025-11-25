use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::{
    error::{EvaluationError, EvaluationErrorType, VeonError},
    parser::{Expr, Stmt},
    token::{TokenType, Value},
};

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), VeonError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(VeonError::EvaluationError(EvaluationError {
            msg: format!("Undefined variable '{name}'"),
            tty: EvaluationErrorType::InvalidOperation,
        }))
    }

    pub fn get(&self, name: &str) -> Result<Value, VeonError> {
        if let Some(value) = self.values.get(name) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        Err(VeonError::EvaluationError(EvaluationError {
            msg: format!("Undefined variable '{name}'"),
            tty: EvaluationErrorType::InvalidOperation,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct VeonFunction {
    pub name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl VeonFunction {
    fn bind(&self, instance: Rc<RefCell<VeonInstance>>) -> Rc<VeonFunction> {
        let mut env = Environment::with_enclosing(self.closure.clone());
        env.define("this".to_string(), Value::Instance(instance));
        Rc::new(VeonFunction {
            name: self.name.clone(),
            params: self.params.clone(),
            body: self.body.clone(),
            closure: Rc::new(RefCell::new(env)),
            is_initializer: self.is_initializer,
        })
    }
}

#[derive(Debug, Clone)]
pub struct VeonClass {
    pub name: String,
    methods: HashMap<String, Rc<VeonFunction>>,
}

#[derive(Debug, Clone)]
pub struct VeonInstance {
    pub class: Rc<VeonClass>,
    fields: HashMap<String, Value>,
}

impl VeonInstance {
    fn new(class: Rc<VeonClass>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    fn get(&self, name: &str) -> Result<Value, VeonError> {
        if let Some(value) = self.fields.get(name) {
            return Ok(value.clone());
        }

        if let Some(method) = self.class.methods.get(name) {
            let bound = method.bind(Rc::new(RefCell::new(self.clone())));
            return Ok(Value::Function(bound));
        }

        Err(VeonError::EvaluationError(EvaluationError {
            msg: format!("Undefined property '{name}'"),
            tty: EvaluationErrorType::InvalidOperation,
        }))
    }

    fn set(&mut self, name: &str, value: Value) {
        self.fields.insert(name.to_string(), value);
    }
}

enum Control {
    Value(Option<Value>),
    Return(Value),
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<Option<Value>, VeonError> {
        let mut last_value = None;
        for statement in statements {
            match self.execute(statement)? {
                Control::Value(value) => last_value = value,
                Control::Return(value) => return Ok(Some(value)),
            }
        }
        Ok(last_value)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<Control, VeonError> {
        match stmt {
            Stmt::Expression(expr) => Ok(Control::Value(Some(self.evaluate(expr)?))),
            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    Value::Null
                };
                self.environment.borrow_mut().define(name.clone(), value);
                Ok(Control::Value(None))
            }
            Stmt::Block(statements) => self
                .execute_block(statements, Environment::with_enclosing(self.environment.clone())),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_val = self.evaluate(condition)?;
                if self.is_truthy(&condition_val) {
                    self.execute(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)
                } else {
                    Ok(Control::Value(None))
                }
            }
            Stmt::While { condition, body } => {
                let mut last = None;
                while {
                    let cond_value = self.evaluate(condition)?;
                    self.is_truthy(&cond_value)
                } {
                    match self.execute(body)? {
                        Control::Value(v) => last = v,
                        Control::Return(v) => return Ok(Control::Return(v)),
                    }
                }
                Ok(Control::Value(last))
            }
            Stmt::Function { name, params, body } => {
                let function = Rc::new(VeonFunction {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.environment.clone(),
                    is_initializer: false,
                });
                self.environment
                    .borrow_mut()
                    .define(name.clone(), Value::Function(function));
                Ok(Control::Value(None))
            }
            Stmt::Return(expr) => {
                let value = if let Some(expr) = expr {
                    self.evaluate(expr)?
                } else {
                    Value::Null
                };
                Ok(Control::Return(value))
            }
            Stmt::Class { name, methods } => {
                self.environment
                    .borrow_mut()
                    .define(name.clone(), Value::Null);

                let mut method_map = HashMap::new();
                for method in methods {
                    if let Stmt::Function { name: mname, params, body } = method {
                        let is_initializer = mname == "init";
                        let function = Rc::new(VeonFunction {
                            name: mname.clone(),
                            params: params.clone(),
                            body: body.clone(),
                            closure: self.environment.clone(),
                            is_initializer,
                        });
                        method_map.insert(mname.clone(), function);
                    }
                }

                let class = Rc::new(VeonClass {
                    name: name.clone(),
                    methods: method_map,
                });
                self.environment
                    .borrow_mut()
                    .assign(name, Value::Class(class))?;
                Ok(Control::Value(None))
            }
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Environment,
    ) -> Result<Control, VeonError> {
        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(environment));
        let mut last = None;
        for statement in statements {
            match self.execute(statement)? {
                Control::Value(v) => last = v,
                Control::Return(v) => {
                    self.environment = previous;
                    return Ok(Control::Return(v));
                }
            }
        }
        self.environment = previous;
        Ok(Control::Value(last))
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, VeonError> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary { operator, right } => {
                let right_val = self.evaluate(right)?;
                match operator {
                    TokenType::Minus => {
                        self.numeric_op(Value::Number(0), right_val, |a, b| a - b)
                    }
                    TokenType::Not => Ok(Value::Boolean(!self.is_truthy(&right_val))),
                    _ => Err(self.runtime_error(
                        "Unsupported unary operator",
                        EvaluationErrorType::InvalidOperation,
                    )),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                match operator {
                    TokenType::Plus => self.add_values(left_val, right_val),
                    TokenType::Minus => self.numeric_op(left_val, right_val, |a, b| a - b),
                    TokenType::Star => self.numeric_op(left_val, right_val, |a, b| a * b),
                    TokenType::Slash => {
                        if right_val == Value::Number(0) {
                            return Err(self.runtime_error(
                                "Divide by zero",
                                EvaluationErrorType::DivideByZero,
                            ));
                        }
                        self.numeric_op(left_val, right_val, |a, b| a / b)
                    }
                    TokenType::Modulo => {
                        if right_val == Value::Number(0) {
                            return Err(self.runtime_error(
                                "Divide by zero",
                                EvaluationErrorType::DivideByZero,
                            ));
                        }
                        self.numeric_op(left_val, right_val, |a, b| a % b)
                    }
                    TokenType::Greater => self.compare(left_val, right_val, |a, b| a > b),
                    TokenType::GreaterEqual => {
                        self.compare(left_val, right_val, |a, b| a >= b)
                    }
                    TokenType::Less => self.compare(left_val, right_val, |a, b| a < b),
                    TokenType::LessEqual => self.compare(left_val, right_val, |a, b| a <= b),
                    TokenType::EqualEqual => Ok(Value::Boolean(left_val == right_val)),
                    TokenType::NotEqual => Ok(Value::Boolean(left_val != right_val)),
                    _ => Err(self.runtime_error(
                        "Unsupported binary operator",
                        EvaluationErrorType::InvalidOperation,
                    )),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate(left)?;
                if matches!(operator, TokenType::Or) {
                    if self.is_truthy(&left_val) {
                        return Ok(left_val);
                    }
                } else if !self.is_truthy(&left_val) {
                    return Ok(left_val);
                }
                self.evaluate(right)
            }
            Expr::Variable(name) => self.environment.borrow().get(name),
            Expr::Assign { name, value } => {
                let val = self.evaluate(value)?;
                self.environment.borrow_mut().assign(name, val.clone())?;
                Ok(val)
            }
            Expr::Array(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(self.evaluate(item)?);
                }
                Ok(Value::Array(values))
            }
            Expr::Index { array, index } => {
                let array_val = self.evaluate(array)?;
                let index_val = self.evaluate(index)?;
                let idx = match index_val {
                    Value::Number(num) if num >= 0 => num as usize,
                    _ => {
                        return Err(self.runtime_error(
                            "Array index must be a non-negative number",
                            EvaluationErrorType::InvalidTypeOperation,
                        ))
                    }
                };

                match array_val {
                    Value::Array(values) => values
                        .get(idx)
                        .cloned()
                        .ok_or_else(|| {
                            self.runtime_error(
                                &format!("Index {idx} out of bounds"),
                                EvaluationErrorType::InvalidOperation,
                            )
                        }),
                    _ => Err(self.runtime_error(
                        "Can only index arrays",
                        EvaluationErrorType::InvalidTypeOperation,
                    )),
                }
            }
            Expr::Call {
                callee,
                arguments,
                ..
            } => {
                let callee_val = self.evaluate(callee)?;
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.evaluate(arg)?);
                }
                self.call_value(callee_val, args)
            }
            Expr::Get { object, name } => {
                let object_val = self.evaluate(object)?;
                if let Value::Instance(instance) = object_val {
                    return instance.borrow().get(name);
                }
                Err(self.runtime_error(
                    "Only instances have properties",
                    EvaluationErrorType::InvalidTypeOperation,
                ))
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                let object_val = self.evaluate(object)?;
                let value_val = self.evaluate(value)?;
                if let Value::Instance(instance) = object_val {
                    instance.borrow_mut().set(name, value_val.clone());
                    return Ok(value_val);
                }
                Err(self.runtime_error(
                    "Only instances have fields",
                    EvaluationErrorType::InvalidTypeOperation,
                ))
            }
            Expr::This => self.environment.borrow().get("this"),
        }
    }

    fn call_value(&mut self, callee: Value, args: Vec<Value>) -> Result<Value, VeonError> {
        match callee {
            Value::Function(func) => self.call_function(func, args),
            Value::Class(class) => {
                let instance = Rc::new(RefCell::new(VeonInstance::new(class.clone())));
                if let Some(initializer) = class.methods.get("init") {
                    let bound = initializer.bind(instance.clone());
                    self.call_function(bound, args)?;
                }
                Ok(Value::Instance(instance))
            }
            _ => Err(self.runtime_error(
                "Can only call functions and classes",
                EvaluationErrorType::InvalidOperation,
            )),
        }
    }

    fn call_function(&mut self, func: Rc<VeonFunction>, args: Vec<Value>) -> Result<Value, VeonError> {
        if args.len() != func.params.len() {
            return Err(self.runtime_error(
                &format!(
                    "Expected {} arguments but got {}",
                    func.params.len(),
                    args.len()
                ),
                EvaluationErrorType::InvalidOperation,
            ));
        }

        let mut env = Environment::with_enclosing(func.closure.clone());
        for (param, arg) in func.params.iter().zip(args.into_iter()) {
            env.define(param.clone(), arg);
        }

        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(env));
        let mut result = Value::Null;
        for stmt in &func.body {
            match self.execute(stmt)? {
                Control::Value(v) => {
                    if let Some(value) = v {
                        result = value;
                    }
                }
                Control::Return(value) => {
                    self.environment = previous;
                    return if func.is_initializer {
                        func.closure
                            .borrow()
                            .get("this")
                            .or_else(|_| Ok(Value::Null))
                    } else {
                        Ok(value)
                    };
                }
            }
        }
        self.environment = previous;
        if func.is_initializer {
            func.closure
                .borrow()
                .get("this")
                .or_else(|_| Ok(Value::Null))
        } else {
            Ok(result)
        }
    }

    fn numeric_op<F>(&self, left: Value, right: Value, f: F) -> Result<Value, VeonError>
    where
        F: FnOnce(isize, isize) -> isize,
    {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
            _ => Err(self.runtime_error(
                "Operands must be numbers",
                EvaluationErrorType::InvalidTypeOperation,
            )),
        }
    }

    fn compare<F>(&self, left: Value, right: Value, f: F) -> Result<Value, VeonError>
    where
        F: FnOnce(isize, isize) -> bool,
    {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(f(a, b))),
            _ => Err(self.runtime_error(
                "Operands must be numbers",
                EvaluationErrorType::InvalidTypeOperation,
            )),
        }
    }

    fn add_values(&self, left: Value, right: Value) -> Result<Value, VeonError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
            (Value::Array(mut a), Value::Array(b)) => {
                a.extend(b);
                Ok(Value::Array(a))
            }
            _ => Err(self.runtime_error(
                "Operands must be two numbers, two strings, or two arrays",
                EvaluationErrorType::InvalidTypeOperation,
            )),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Null | Value::None => false,
            Value::Number(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Array(items) => !items.is_empty(),
            Value::Function(_) | Value::Class(_) | Value::Instance(_) => true,
        }
    }

    fn runtime_error(&self, msg: &str, tty: EvaluationErrorType) -> VeonError {
        VeonError::EvaluationError(EvaluationError {
            msg: msg.to_string(),
            tty,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, scanner::Scanner};

    use super::*;

    fn interpret_source(source: &str) -> Option<Value> {
        let mut scanner = Scanner::new(source.to_string());
        let tokens = scanner.tokenize().expect("tokenize");
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().expect("parse");
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&statements).expect("interpret")
    }

    #[test]
    fn interpret_arithmetic_and_assignment() {
        let result = interpret_source("let x = 2 + 3 * 4; x = x - 5; x;");
        assert_eq!(result, Some(Value::Number(9)));
    }

    #[test]
    fn interpret_arrays_and_indexing() {
        let result = interpret_source("let items = [1, 2, 3]; items[1];");
        assert_eq!(result, Some(Value::Number(2)));
    }

    #[test]
    fn interpret_string_concatenation() {
        let result = interpret_source("\"hello\" + \" world\";");
        assert_eq!(result, Some(Value::String("hello world".to_string())));
    }

    #[test]
    fn interpret_functions_and_loops() {
        let result = interpret_source(
            "fun add(a, b) { return a + b; } let total = 0; let i = 0; while (i < 3) { total = add(total, i); i = i + 1; } total;",
        );
        assert_eq!(result, Some(Value::Number(3)));
    }

    #[test]
    fn interpret_classes_and_this() {
        let result = interpret_source(
            "class Counter { fun init(start) { this.value = start; } fun inc() { this.value = this.value + 1; return this.value; } } let c = Counter(1); c.inc();",
        );
        assert_eq!(result, Some(Value::Number(2)));
    }
}
