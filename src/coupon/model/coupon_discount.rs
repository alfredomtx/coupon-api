use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct CouponDiscount(i32);

impl CouponDiscount {
    pub fn parse(d: i32) -> Result<Self, String> {
        if (d > 90){
            return Err("Discount cannot be higher than 90.".to_string());
        }
        if (d < 0){
            return Err("Discount cannot be less than 0.".to_string());
        }

        return Ok( Self(d) );
    }
}

impl AsRef<i32> for CouponDiscount {
    // The caller gets a shared reference to the inner i32.
    // This gives the caller **read-only** access,
    // they have no way to compromise our invariants!
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::CouponDiscount;
    use claim::{assert_err, assert_ok};

    #[test]
    fn valid_discount_is_accepted(){
        assert_ok!(CouponDiscount::parse(10));
    }

    #[test]
    fn discount_higher_than_90_is_rejected(){
        assert_ok!(CouponDiscount::parse(90));
        assert_err!(CouponDiscount::parse(91));
    }

    #[test]
    fn discount_less_than_0_is_rejected(){
        assert_ok!(CouponDiscount::parse(0));
        assert_err!(CouponDiscount::parse(-1));
    }
}