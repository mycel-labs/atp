use ethers_core::types::U256;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{CaipError, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Money {
    /// The raw amount without decimal point (e.g., wei for ETH, lamports for SOL)
    #[serde(
        serialize_with = "serialize_u256",
        deserialize_with = "deserialize_u256"
    )]
    pub amount: U256,
    /// Number of decimal places for this currency
    pub decimals: u8,
}

// Custom serialization for U256
fn serialize_u256<S>(value: &U256, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn deserialize_u256<'de, D>(deserializer: D) -> std::result::Result<U256, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    U256::from_dec_str(&s).map_err(serde::de::Error::custom)
}

impl Money {
    /// Creates a new Money instance with the specified amount and decimal precision.
    /// * `amount` - The raw amount as U256 (e.g., wei for ETH, lamports for SOL)
    /// * `decimals` - Number of decimal places for this currency (max 77)
    /// * `Result<Self>` - A new Money instance if valid, or CaipError if decimals > 77
    /// use ethers_core::types::U256;
    /// let eth = Money::new(U256::from_dec_str("1000000000000000000").unwrap(), 18).unwrap();
    /// let usdc = Money::new(U256::from(1000000), 6).unwrap();
    pub fn new(amount: U256, decimals: u8) -> Result<Self> {
        if decimals > 77 {
            return Err(CaipError::DecimalOverflow {
                max: 77,
                got: decimals,
            });
        }
        Ok(Self { amount, decimals })
    }

    /// Creates a Money instance with zero amount and specified decimal precision.
    /// * `decimals` - Number of decimal places for this currency
    /// * `Result<Self>` - A zero Money instance
    /// let zero_eth = Money::zero(18).unwrap();
    /// assert!(zero_eth.is_zero());
    pub fn zero(decimals: u8) -> Result<Self> {
        Self::new(U256::zero(), decimals)
    }

    /// Creates Money from a raw amount string representation.
    /// * `decimals` - Number of decimal places for this currency
    /// let eth = Money::from_raw("1000000000000000000", 18).unwrap();
    /// assert_eq!(eth.to_decimal_string(), "1");
    pub fn from_raw(s: &str, decimals: u8) -> Result<Self> {
        let amount = U256::from_dec_str(s).map_err(|_| CaipError::InvalidAmount(s.to_string()))?;
        Self::new(amount, decimals)
    }

    /// Creates Money from a human-readable decimal string.
    /// * `decimals` - Number of decimal places for this currency
    /// * `Result<Self>` - A Money instance if parsing succeeds, or CaipError if invalid format
    /// let eth = Money::from_decimal_str("1.5", 18).unwrap();
    /// let usdc = Money::from_decimal_str("100.50", 6).unwrap();
    pub fn from_decimal_str(value: &str, decimals: u8) -> Result<Self> {
        // Handle empty or invalid input
        if value.trim().is_empty() {
            return Err(CaipError::InvalidAmount("Empty amount".to_string()));
        }

        // Split into integer and fractional parts
        let parts: Vec<&str> = value.split('.').collect();
        if parts.len() > 2 {
            return Err(CaipError::InvalidAmount(format!(
                "Invalid decimal format: {}",
                value
            )));
        }

        // Handle case of just "." with no integers on either side
        if parts.len() == 2 && parts[0].is_empty() && parts[1].is_empty() {
            return Err(CaipError::InvalidAmount(format!(
                "Invalid decimal format: {}",
                value
            )));
        }

        // Parse integer part
        let integer_part = U256::from_dec_str(parts[0])
            .map_err(|_| CaipError::InvalidAmount(format!("Invalid integer part: {}", parts[0])))?;

        // Handle fractional part
        let fractional_part = if parts.len() == 2 {
            let fraction_str = parts[1];
            if fraction_str.len() > decimals as usize {
                return Err(CaipError::InvalidAmount(format!(
                    "Too many decimal places: {} (max: {})",
                    fraction_str.len(),
                    decimals
                )));
            }

            // Pad with zeros to match decimals
            let padded = format!("{:0<width$}", fraction_str, width = decimals as usize);
            U256::from_dec_str(&padded).map_err(|_| {
                CaipError::InvalidAmount(format!("Invalid fractional part: {}", fraction_str))
            })?
        } else {
            U256::zero()
        };

        // Calculate total amount
        let multiplier = U256::from(10u64).pow(U256::from(decimals));
        let total = integer_part * multiplier + fractional_part;

        Ok(Self {
            amount: total,
            decimals,
        })
    }

    /// Alias for from_decimal_str for backwards compatibility.
    /// * `s` - Decimal string representation
    /// * `decimals` - Number of decimal places for this currency
    /// * `Result<Self>` - A Money instance if parsing succeeds
    pub fn from_human_readable(s: &str, decimals: u8) -> Result<Self> {
        Self::from_decimal_str(s, decimals)
    }

    /// Converts to a human-readable decimal string representation.
    /// let eth = Money::from_decimal_str("1.5", 18).unwrap();
    /// assert_eq!(eth.to_decimal_string(), "1.5");
    pub fn to_decimal_string(&self) -> String {
        if self.decimals == 0 {
            return self.amount.to_string();
        }

        let divisor = U256::from(10u64).pow(U256::from(self.decimals));
        let integer_part = self.amount / divisor;
        let fractional_part = self.amount % divisor;

        if fractional_part.is_zero() {
            integer_part.to_string()
        } else {
            let fractional_str = format!(
                "{:0>width$}",
                fractional_part,
                width = self.decimals as usize
            );
            // Remove trailing zeros
            let trimmed = fractional_str.trim_end_matches('0');
            format!("{}.{}", integer_part, trimmed)
        }
    }

    /// Alias for to_decimal_string for backwards compatibility.
    pub fn to_human_readable(&self) -> String {
        self.to_decimal_string()
    }

    /// Converts to floating point representation (use with caution due to precision loss).
    /// let eth = Money::from_decimal_str("1.5", 18).unwrap();
    /// assert_eq!(eth.to_f64(), 1.5);
    pub fn to_f64(&self) -> f64 {
        let divisor = 10f64.powi(self.decimals as i32);
        // Note: This may lose precision for very large numbers
        let amount_str = self.amount.to_string();
        let amount_f64 = amount_str.parse::<f64>().unwrap_or(f64::MAX);
        amount_f64 / divisor
    }

    pub fn raw_amount(&self) -> U256 {
        self.amount
    }

    /// Alias for raw_amount.
    pub fn to_base_units(&self) -> U256 {
        self.amount
    }

    /// * `bool` - True if the amount is zero
    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    /// * `Result<Money>` - Sum of the two amounts, or error if different decimals
    /// let a = Money::from_decimal_str("1.5", 18).unwrap();
    /// let b = Money::from_decimal_str("2.5", 18).unwrap();
    /// let sum = a.add(&b).unwrap();
    pub fn add(&self, other: &Money) -> Result<Money> {
        if self.decimals != other.decimals {
            return Err(CaipError::InvalidAmount(format!(
                "Cannot add amounts with different decimals: {} vs {}",
                self.decimals, other.decimals
            )));
        }

        Ok(Money {
            amount: self.amount + other.amount,
            decimals: self.decimals,
        })
    }

    /// Alias for add with overflow checking.
    /// * `Result<Money>` - Sum of the two amounts
    pub fn checked_add(&self, other: &Money) -> Result<Money> {
        self.add(other)
    }

    /// Subtracts two Money values (must have same decimal precision).
    /// * `Result<Money>` - Difference of the two amounts, or error if insufficient funds or different decimals
    /// let a = Money::from_decimal_str("2.5", 18).unwrap();
    /// let b = Money::from_decimal_str("1.5", 18).unwrap();
    /// let diff = a.sub(&b).unwrap();
    /// assert_eq!(diff.to_decimal_string(), "1");
    pub fn sub(&self, other: &Money) -> Result<Money> {
        if self.decimals != other.decimals {
            return Err(CaipError::InvalidAmount(format!(
                "Cannot subtract amounts with different decimals: {} vs {}",
                self.decimals, other.decimals
            )));
        }

        if self.amount < other.amount {
            return Err(CaipError::InvalidAmount(format!(
                "Insufficient amount: {} < {}",
                self.to_decimal_string(),
                other.to_decimal_string()
            )));
        }

        Ok(Money {
            amount: self.amount - other.amount,
            decimals: self.decimals,
        })
    }

    /// Alias for sub with underflow checking.
    /// * `Result<Money>` - Difference of the two amounts
    pub fn checked_sub(&self, other: &Money) -> Result<Money> {
        self.sub(other)
    }

    /// Multiplies by a scalar value.
    /// let eth = Money::from_decimal_str("1.5", 18).unwrap();
    /// assert_eq!(doubled.to_decimal_string(), "3");
    pub fn mul(&self, scalar: u64) -> Money {
        Money {
            amount: self.amount * U256::from(scalar),
            decimals: self.decimals,
        }
    }

    /// Multiplies by a U256 scalar value.
    pub fn mul_u256(&self, scalar: U256) -> Money {
        Money {
            amount: self.amount * scalar,
            decimals: self.decimals,
        }
    }

    /// Divides by a scalar value.
    /// * `Result<Money>` - The divided amount, or error if division by zero
    /// let eth = Money::from_decimal_str("3", 18).unwrap();
    /// let halved = eth.div(2).unwrap();
    /// assert_eq!(halved.to_decimal_string(), "1.5");
    pub fn div(&self, divisor: u64) -> Result<Money> {
        if divisor == 0 {
            return Err(CaipError::InvalidAmount("Division by zero".to_string()));
        }

        Ok(Money {
            amount: self.amount / U256::from(divisor),
            decimals: self.decimals,
        })
    }

    /// Divides by a U256 scalar value.
    /// * `Result<Money>` - The divided amount, or error if division by zero
    pub fn div_u256(&self, divisor: U256) -> Result<Money> {
        if divisor.is_zero() {
            return Err(CaipError::InvalidAmount("Division by zero".to_string()));
        }

        Ok(Money {
            amount: self.amount / divisor,
            decimals: self.decimals,
        })
    }

    /// Calculates a percentage of the amount.
    /// let eth = Money::from_decimal_str("100", 18).unwrap();
    /// assert_eq!(fee.to_decimal_string(), "3");
    pub fn percentage(&self, percent: u64) -> Money {
        Money {
            amount: self.amount * U256::from(percent) / U256::from(100),
            decimals: self.decimals,
        }
    }

    /// Calculates basis points of the amount.
    /// let eth = Money::from_decimal_str("100", 18).unwrap();
    /// assert_eq!(fee.to_decimal_string(), "2.5");
    pub fn basis_points(&self, bps: u64) -> Money {
        Money {
            amount: self.amount * U256::from(bps) / U256::from(10000),
            decimals: self.decimals,
        }
    }

    /// Compares if this amount is greater than another (must have same decimals).
    /// * `Result<bool>` - True if this amount is greater, or error if different decimals
    pub fn gt(&self, other: &Money) -> Result<bool> {
        if self.decimals != other.decimals {
            return Err(CaipError::InvalidAmount(format!(
                "Cannot compare amounts with different decimals: {} vs {}",
                self.decimals, other.decimals
            )));
        }
        Ok(self.amount > other.amount)
    }

    /// Compares if this amount is greater than or equal to another.
    /// * `Result<bool>` - True if this amount is greater than or equal, or error if different decimals
    pub fn gte(&self, other: &Money) -> Result<bool> {
        if self.decimals != other.decimals {
            return Err(CaipError::InvalidAmount(format!(
                "Cannot compare amounts with different decimals: {} vs {}",
                self.decimals, other.decimals
            )));
        }
        Ok(self.amount >= other.amount)
    }

    /// Compares if this amount is less than another.
    /// * `Result<bool>` - True if this amount is less, or error if different decimals
    pub fn lt(&self, other: &Money) -> Result<bool> {
        if self.decimals != other.decimals {
            return Err(CaipError::InvalidAmount(format!(
                "Cannot compare amounts with different decimals: {} vs {}",
                self.decimals, other.decimals
            )));
        }
        Ok(self.amount < other.amount)
    }

    /// Compares if this amount is less than or equal to another.
    /// * `Result<bool>` - True if this amount is less than or equal, or error if different decimals
    pub fn lte(&self, other: &Money) -> Result<bool> {
        if self.decimals != other.decimals {
            return Err(CaipError::InvalidAmount(format!(
                "Cannot compare amounts with different decimals: {} vs {}",
                self.decimals, other.decimals
            )));
        }
        Ok(self.amount <= other.amount)
    }

    /// * `Result<Money>` - The smaller of the two amounts, or error if different decimals
    pub fn min(&self, other: &Money) -> Result<Money> {
        if self.lte(other)? {
            Ok(self.clone())
        } else {
            Ok(other.clone())
        }
    }

    /// * `Result<Money>` - The larger of the two amounts, or error if different decimals
    pub fn max(&self, other: &Money) -> Result<Money> {
        if self.gte(other)? {
            Ok(self.clone())
        } else {
            Ok(other.clone())
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_decimal_string())
    }
}

impl FromStr for Money {
    type Err = CaipError;

    fn from_str(s: &str) -> Result<Self> {
        // Default to 18 decimals if not specified
        // In practice, you'd want to know the currency type
        Self::from_decimal_str(s, 18)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_core::types::U256;
    use std::str::FromStr;

    #[test]
    fn test_money_creation() {
        // Test new
        let money = Money::new(U256::from(1000), 18).unwrap();
        assert_eq!(money.amount, U256::from(1000));
        assert_eq!(money.decimals, 18);

        // Test zero
        let zero = Money::zero(6).unwrap();
        assert!(zero.is_zero());
        assert_eq!(zero.decimals, 6);

        // Test decimal overflow
        assert!(Money::new(U256::from(1000), 78).is_err());
    }

    #[test]
    fn test_from_raw() {
        let money = Money::from_raw("1000000000000000000", 18).unwrap();
        assert_eq!(money.to_decimal_string(), "1");

        let money = Money::from_raw("5000000", 6).unwrap();
        assert_eq!(money.to_decimal_string(), "5");

        // Test invalid raw string
        assert!(Money::from_raw("invalid", 18).is_err());
    }

    #[test]
    fn test_from_decimal_str() {
        // Test whole numbers
        let money = Money::from_decimal_str("100", 18).unwrap();
        assert_eq!(money.to_decimal_string(), "100");
        assert_eq!(
            money.amount,
            U256::from_dec_str("100000000000000000000").unwrap()
        );

        // Test decimals
        let money = Money::from_decimal_str("1.5", 18).unwrap();
        assert_eq!(money.to_decimal_string(), "1.5");
        assert_eq!(
            money.amount,
            U256::from_dec_str("1500000000000000000").unwrap()
        );

        // Test with trailing zeros
        let money = Money::from_decimal_str("1.500", 18).unwrap();
        assert_eq!(money.to_decimal_string(), "1.5");

        // Test small decimals
        let money = Money::from_decimal_str("0.000001", 18).unwrap();
        assert_eq!(money.to_decimal_string(), "0.000001");

        // Test zero
        let money = Money::from_decimal_str("0", 18).unwrap();
        assert!(money.is_zero());

        // Test with different decimals
        let money = Money::from_decimal_str("123.456", 6).unwrap();
        assert_eq!(money.to_decimal_string(), "123.456");
        assert_eq!(money.amount, U256::from(123456000));
    }

    #[test]
    fn test_from_decimal_str_errors() {
        // Empty string
        assert!(Money::from_decimal_str("", 18).is_err());
        assert!(Money::from_decimal_str("  ", 18).is_err());

        // Invalid format
        assert!(Money::from_decimal_str("1.2.3", 18).is_err());
        assert!(Money::from_decimal_str("1..2", 18).is_err());
        assert!(Money::from_decimal_str(".", 18).is_err());

        // Too many decimals
        assert!(Money::from_decimal_str("1.1234567", 6).is_err());
        assert!(Money::from_decimal_str("0.0000001", 6).is_err());

        // Invalid characters
        assert!(Money::from_decimal_str("1a.5", 18).is_err());
        assert!(Money::from_decimal_str("1.5b", 18).is_err());
    }

    #[test]
    fn test_to_decimal_string() {
        // Test whole numbers
        let money = Money::new(U256::from_dec_str("1000000000000000000").unwrap(), 18).unwrap();
        assert_eq!(money.to_decimal_string(), "1");

        // Test with decimals
        let money = Money::new(U256::from_dec_str("1500000000000000000").unwrap(), 18).unwrap();
        assert_eq!(money.to_decimal_string(), "1.5");

        // Test trailing zeros removed
        let money = Money::new(U256::from_dec_str("1230000000000000000").unwrap(), 18).unwrap();
        assert_eq!(money.to_decimal_string(), "1.23");

        // Test zero decimals
        let money = Money::new(U256::from(12345), 0).unwrap();
        assert_eq!(money.to_decimal_string(), "12345");

        // Test very small amounts
        let money = Money::new(U256::from(1), 18).unwrap();
        assert_eq!(money.to_decimal_string(), "0.000000000000000001");
    }

    #[test]
    fn test_to_f64() {
        let money = Money::from_decimal_str("123.456", 6).unwrap();
        assert!((money.to_f64() - 123.456).abs() < 0.000001);

        let money = Money::from_decimal_str("0.000001", 6).unwrap();
        assert!((money.to_f64() - 0.000001).abs() < 0.0000001);

        // Test large number (may lose precision)
        let money = Money::from_decimal_str("1000000", 18).unwrap();
        assert!(money.to_f64() > 999999.0);
    }

    #[test]
    fn test_arithmetic_add() {
        let a = Money::from_decimal_str("10.5", 6).unwrap();
        let b = Money::from_decimal_str("5.25", 6).unwrap();

        let sum = a.add(&b).unwrap();
        assert_eq!(sum.to_decimal_string(), "15.75");

        // Test checked_add alias
        let sum2 = a.checked_add(&b).unwrap();
        assert_eq!(sum2.to_decimal_string(), "15.75");

        // Test adding zero
        let zero = Money::zero(6).unwrap();
        let sum3 = a.add(&zero).unwrap();
        assert_eq!(sum3.to_decimal_string(), "10.5");

        // Test different decimals error
        let c = Money::from_decimal_str("1", 18).unwrap();
        assert!(a.add(&c).is_err());
    }

    #[test]
    fn test_arithmetic_sub() {
        let a = Money::from_decimal_str("10.5", 6).unwrap();
        let b = Money::from_decimal_str("5.25", 6).unwrap();

        let diff = a.sub(&b).unwrap();
        assert_eq!(diff.to_decimal_string(), "5.25");

        // Test checked_sub alias
        let diff2 = a.checked_sub(&b).unwrap();
        assert_eq!(diff2.to_decimal_string(), "5.25");

        // Test subtracting same amount
        let diff3 = a.sub(&a).unwrap();
        assert!(diff3.is_zero());

        // Test underflow error
        assert!(b.sub(&a).is_err());

        // Test different decimals error
        let c = Money::from_decimal_str("1", 18).unwrap();
        assert!(a.sub(&c).is_err());
    }

    #[test]
    fn test_arithmetic_mul() {
        let a = Money::from_decimal_str("10.5", 6).unwrap();

        let doubled = a.mul(2);
        assert_eq!(doubled.to_decimal_string(), "21");

        let tripled = a.mul(3);
        assert_eq!(tripled.to_decimal_string(), "31.5");

        let zero_mul = a.mul(0);
        assert!(zero_mul.is_zero());

        // Test mul_u256
        let large_multiplier = U256::from(1000);
        let large_result = a.mul_u256(large_multiplier);
        assert_eq!(large_result.to_decimal_string(), "10500");
    }

    #[test]
    fn test_arithmetic_div() {
        let a = Money::from_decimal_str("10.5", 6).unwrap();

        let halved = a.div(2).unwrap();
        assert_eq!(halved.to_decimal_string(), "5.25");

        let third = a.div(3).unwrap();
        assert_eq!(third.to_decimal_string(), "3.5");

        // Test division by zero
        assert!(a.div(0).is_err());

        // Test div_u256
        let divisor = U256::from(4);
        let quarter = a.div_u256(divisor).unwrap();
        assert_eq!(quarter.to_decimal_string(), "2.625");

        // Test division by zero with U256
        assert!(a.div_u256(U256::zero()).is_err());
    }

    #[test]
    fn test_percentage() {
        let amount = Money::from_decimal_str("100", 6).unwrap();

        let ten_percent = amount.percentage(10);
        assert_eq!(ten_percent.to_decimal_string(), "10");

        let half_percent = amount.percentage(50);
        assert_eq!(half_percent.to_decimal_string(), "50");

        let zero_percent = amount.percentage(0);
        assert!(zero_percent.is_zero());

        let hundred_percent = amount.percentage(100);
        assert_eq!(hundred_percent.to_decimal_string(), "100");

        // Test with decimals
        let amount2 = Money::from_decimal_str("33.33", 6).unwrap();
        let third = amount2.percentage(33);
        assert_eq!(third.to_decimal_string(), "10.9989");
    }

    #[test]
    fn test_basis_points() {
        let amount = Money::from_decimal_str("1000", 6).unwrap();

        // 100 bps = 1%
        let one_percent = amount.basis_points(100);
        assert_eq!(one_percent.to_decimal_string(), "10");

        // 30 bps = 0.3%
        let fee = amount.basis_points(30);
        assert_eq!(fee.to_decimal_string(), "3");

        // 1 bps = 0.01%
        let tiny_fee = amount.basis_points(1);
        assert_eq!(tiny_fee.to_decimal_string(), "0.1");

        // 10000 bps = 100%
        let all = amount.basis_points(10000);
        assert_eq!(all.to_decimal_string(), "1000");
    }

    #[test]
    fn test_comparisons() {
        let a = Money::from_decimal_str("10.5", 6).unwrap();
        let b = Money::from_decimal_str("5.25", 6).unwrap();
        let c = Money::from_decimal_str("10.5", 6).unwrap();

        // Greater than
        assert!(a.gt(&b).unwrap());
        assert!(!b.gt(&a).unwrap());
        assert!(!a.gt(&c).unwrap());

        // Greater than or equal
        assert!(a.gte(&b).unwrap());
        assert!(!b.gte(&a).unwrap());
        assert!(a.gte(&c).unwrap());

        // Less than
        assert!(!a.lt(&b).unwrap());
        assert!(b.lt(&a).unwrap());
        assert!(!a.lt(&c).unwrap());

        // Less than or equal
        assert!(!a.lte(&b).unwrap());
        assert!(b.lte(&a).unwrap());
        assert!(a.lte(&c).unwrap());

        // Test different decimals error
        let d = Money::from_decimal_str("10", 18).unwrap();
        assert!(a.gt(&d).is_err());
        assert!(a.gte(&d).is_err());
        assert!(a.lt(&d).is_err());
        assert!(a.lte(&d).is_err());
    }

    #[test]
    fn test_min_max() {
        let a = Money::from_decimal_str("10.5", 6).unwrap();
        let b = Money::from_decimal_str("5.25", 6).unwrap();
        let c = Money::from_decimal_str("15.75", 6).unwrap();

        let min_ab = a.min(&b).unwrap();
        assert_eq!(min_ab.to_decimal_string(), "5.25");

        let min_bc = b.min(&c).unwrap();
        assert_eq!(min_bc.to_decimal_string(), "5.25");

        let max_ab = a.max(&b).unwrap();
        assert_eq!(max_ab.to_decimal_string(), "10.5");

        let max_bc = b.max(&c).unwrap();
        assert_eq!(max_bc.to_decimal_string(), "15.75");

        // Test with equal values
        let d = Money::from_decimal_str("10.5", 6).unwrap();
        let min_ad = a.min(&d).unwrap();
        assert_eq!(min_ad.to_decimal_string(), "10.5");

        // Test different decimals error
        let e = Money::from_decimal_str("10", 18).unwrap();
        assert!(a.min(&e).is_err());
        assert!(a.max(&e).is_err());
    }

    #[test]
    fn test_display_and_from_str() {
        let money = Money::from_decimal_str("123.456", 18).unwrap();
        assert_eq!(format!("{}", money), "123.456");

        // Test FromStr with default 18 decimals
        let money2 = Money::from_str("123.456").unwrap();
        assert_eq!(money2.to_decimal_string(), "123.456");
        assert_eq!(money2.decimals, 18);

        // Test FromStr error
        assert!(Money::from_str("invalid").is_err());
    }

    #[test]
    fn test_backwards_compatibility() {
        // Test from_human_readable alias
        let money = Money::from_human_readable("123.45", 6).unwrap();
        assert_eq!(money.to_decimal_string(), "123.45");

        // Test to_human_readable alias
        assert_eq!(money.to_human_readable(), "123.45");

        // Test to_base_units alias
        assert_eq!(money.to_base_units(), money.raw_amount());
    }

    #[test]
    fn test_edge_cases() {
        // Very large number
        let large = Money::from_decimal_str(
            "115792089237316195423570985008687907853269984665640564039457.584007913129639935",
            18,
        )
        .unwrap();
        assert!(!large.is_zero());

        // Very small but non-zero
        let tiny = Money::new(U256::from(1), 18).unwrap();
        assert!(!tiny.is_zero());
        assert_eq!(tiny.to_decimal_string(), "0.000000000000000001");

        // Exactly zero
        let zero = Money::new(U256::zero(), 18).unwrap();
        assert!(zero.is_zero());
        assert_eq!(zero.to_decimal_string(), "0");
    }

    #[test]
    fn test_serialization() {
        let money = Money::from_decimal_str("123.456", 6).unwrap();

        // Serialize to JSON
        let json = serde_json::to_string(&money).unwrap();
        assert!(json.contains("\"123456000\""));
        assert!(json.contains("\"decimals\":6"));

        // Deserialize from JSON
        let deserialized: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.to_decimal_string(), "123.456");
        assert_eq!(deserialized.decimals, 6);
    }

    #[test]
    fn test_real_world_scenarios() {
        // ETH gas calculation
        let gas_price = Money::from_decimal_str("0.00000003", 18).unwrap(); // 30 Gwei
        let gas_limit = 21000;
        let tx_cost = gas_price.mul(gas_limit);
        assert_eq!(tx_cost.to_decimal_string(), "0.00063");

        // DeFi swap with 0.3% fee
        let input = Money::from_decimal_str("1000", 6).unwrap(); // 1000 USDC
        let fee = input.basis_points(30); // 0.3%
        let output = input.sub(&fee).unwrap();
        assert_eq!(fee.to_decimal_string(), "3");
        assert_eq!(output.to_decimal_string(), "997");

        // Token amount with slippage tolerance
        let expected = Money::from_decimal_str("100", 18).unwrap();
        let slippage = expected.percentage(1); // 1% slippage
        let min_output = expected.sub(&slippage).unwrap();
        assert_eq!(min_output.to_decimal_string(), "99");
    }
}
