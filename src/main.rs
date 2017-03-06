extern crate rand;

use rand::Rng;
use std::iter::Peekable;
use std::fmt;

struct DiceRoll { count: u32, sides: u32 }
struct Scalar { value: i32 }
struct Add { lhs: Box<Expression>, rhs: Box<Expression> }
struct Subtract { lhs: Box<Expression>, rhs: Box<Expression> }

pub struct ParseError {
    message: &'static str,
}

pub trait Expression : fmt::Display {
    fn get_value(&self) -> i32;
}

impl fmt::Display for Add {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} + {}", self.lhs, self.rhs)
    }
}

impl Expression for Add {
    fn get_value(&self) -> i32 {
        self.lhs.get_value() + self.rhs.get_value()
    }
}

impl fmt::Display for Subtract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.lhs, self.rhs)
    }
}

impl Expression for Subtract {
    fn get_value(&self) -> i32 {
        self.lhs.get_value() - self.rhs.get_value()
    }
}

impl fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.count {
            1 => write!(f, "d{}", self.sides),
            _ => write!(f, "{}d{}", self.count, self.sides),
        }
    }
}

impl Expression for DiceRoll {
    fn get_value(&self) -> i32 {
        let mut sum = 0;
        for _ in 0..self.count {
            sum += roll_die(self.sides);
        }

        return sum as i32;
    }
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl Expression for Scalar {
    fn get_value(&self) -> i32 {
        return self.value;
    }
}

fn roll_die(sides: u32) -> u32
{
    return (rand::thread_rng().gen::<u32>() % sides) + 1;
}

fn read_u32<T>(roll_def: &mut Peekable<T>, default: u32) -> u32
    where T: Iterator<Item=char> {
    let mut nums_found = false;
    let mut result = 0;
    while match roll_def.peek() {
        Some(&'0'...'9') => true,
        _ => false
    } {
        nums_found = true;
        result *= 10;
        result += (roll_def.next().unwrap() as u32) - ('0' as u32);
    }

    return if nums_found { result } else { default };
}

fn read_operand<T>(chars: &mut Peekable<T>) -> Option<Box<Expression>>
    where T: Iterator<Item=char> {
    match chars.peek() {
        Some(&'0'...'9') | Some(&'d') => {},
        _ => return None,
    }

    let die_count = read_u32(chars, 1);
    match chars.peek() {
        Some(&'d') => {
            chars.next();
            let sides_count = read_u32(chars, 6);
            return Some(Box::new(DiceRoll { count: die_count, sides: sides_count }));
        },
        _ => {},
    }

    return Some(Box::new(Scalar{ value: die_count as i32 }));
}

fn chomp<T>(chars: &mut Peekable<T>) where T: Iterator<Item=char>
{
    while chars.peek().unwrap_or(&'_').is_whitespace() {
        chars.next();
    }
}

pub fn roll(roll_def: &str) -> Result<Box<Expression>, ParseError>
{
    let mut chars = roll_def.chars().peekable();
    chomp(&mut chars);
    let mut result = match read_operand(&mut chars) {
        Some(x) => x,
        None => return Err(ParseError{ message: "Invalid roll definition"}),
    };
    while chars.peek().is_some() {
        chomp(&mut chars);
        match chars.next() {
            Some(operator) => {
                chomp(&mut chars);
                if let Some(rhs) = read_operand(&mut chars) {
                    result = match operator {
                        '+' => Box::new(Add { lhs: result, rhs: rhs }),
                        '-' => Box::new(Subtract { lhs: result, rhs: rhs }),
                        _ => {
                            return Err(ParseError{ message: "Invalid roll definition"});
                        }
                    };
                } else {
                    return Err(ParseError { message: "Missing operand" })
                }
            },
            None => {
                return Ok(result);
            },
        }
    };

    return Ok(result);
}

fn main()
{
    let request: String;
    if std::env::args().len() == 1 {
        request = String::from("d");
    } else {
        request = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    }
    match roll(&request) {
        Ok(result) => println!("{}", result.get_value()),
        Err(err) => {
            println!("ERROR: {}", err.message);
            std::process::exit(-1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        for &(input, output) in [
            ("d", Some("d6")),
            ("d6", Some("d6")),
            ("3d", Some("3d6")),
            ("d12", Some("d12")),
            ("   d12", Some("d12")),
            ("d12 + 52", Some("d12 + 52")),
            ("d12 - 8", Some("d12 - 8")),
            ("3d12 - 8 + 10d8", Some("3d12 - 8 + 10d8")),
        ].into_iter() {
            let result = roll(input);
            println!("testing: {}", input);
            assert_eq!(result.is_ok(), output.is_some());
            if output.is_some() {
                assert_eq!(result.ok().unwrap().to_string(), output.unwrap());
            }
            println!("\tOK");
        }
    }
}
