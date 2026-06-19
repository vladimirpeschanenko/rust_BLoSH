use rust_decimal::Decimal;

#[derive(Debug, PartialEq)]
pub struct Bid {
    price: Decimal,
    quantity: u64,
}

#[derive(Debug, PartialEq)]
pub struct Ask {
    price: Decimal,
    quantity: u64,
}

pub fn get_best_bids_and_asks_from_stream(stream: Vec<&[u8]>) -> (Vec<Bid>, Vec<Ask>) {
    todo!();
}

#[cfg(test)]
mod tests {
    use crate::{
        assignment_two_data::{get_best_bids_and_asks_from_stream, Ask, Bid},
        data_generator::{generate_stream, InputLevel},
    };
    use rust_decimal_macros::dec;

    #[test]
    fn test_data_snapshot() {
        let stream = generate_stream(InputLevel::One);

        let (best_bids, best_asks) = get_best_bids_and_asks_from_stream(stream);

        assert_eq!(
            best_bids,
            vec![
                Bid {
                    price: dec!(0.0024),
                    quantity: 24
                },
                Bid {
                    price: dec!(0.0023),
                    quantity: 11
                },
                Bid {
                    price: dec!(0.0022),
                    quantity: 22
                },
                Bid {
                    price: dec!(0.0021),
                    quantity: 87
                },
                Bid {
                    price: dec!(0.0020),
                    quantity: 24
                }
            ]
        );

        assert_eq!(
            best_asks,
            vec![
                Ask {
                    price: dec!(0.0026),
                    quantity: 10
                },
                Ask {
                    price: dec!(0.0027),
                    quantity: 4
                },
                Ask {
                    price: dec!(0.0028),
                    quantity: 51
                },
                Ask {
                    price: dec!(0.0029),
                    quantity: 20
                },
                Ask {
                    price: dec!(0.0030),
                    quantity: 1
                }
            ]
        );
    }

    #[test]
    fn test_data_without_final_trade() {
        let stream = generate_stream(InputLevel::Two);

        let (best_bids, best_asks) = get_best_bids_and_asks_from_stream(stream);

        assert_eq!(
            best_bids,
            vec![
                Bid {
                    price: dec!(0.0025),
                    quantity: 19
                },
                Bid {
                    price: dec!(0.0024),
                    quantity: 22
                },
                Bid {
                    price: dec!(0.0023),
                    quantity: 13
                },
                Bid {
                    price: dec!(0.0022),
                    quantity: 27
                },
                Bid {
                    price: dec!(0.0020),
                    quantity: 1230
                }
            ]
        );

        assert_eq!(
            best_asks,
            vec![
                Ask {
                    price: dec!(0.0027),
                    quantity: 4
                },
                Ask {
                    price: dec!(0.0028),
                    quantity: 51
                },
                Ask {
                    price: dec!(0.0029),
                    quantity: 20
                },
                Ask {
                    price: dec!(0.0033),
                    quantity: 44
                },
                Ask {
                    price: dec!(0.0039),
                    quantity: 9
                }
            ]
        );
    }

    #[test]
    fn test_data_with_final_trade() {
        let stream = generate_stream(InputLevel::Full);

        let (best_bids, best_asks) = get_best_bids_and_asks_from_stream(stream);

        assert_eq!(
            best_bids,
            vec![
                Bid {
                    price: dec!(0.0024),
                    quantity: 5
                },
                Bid {
                    price: dec!(0.0023),
                    quantity: 13
                },
                Bid {
                    price: dec!(0.0022),
                    quantity: 27
                },
                Bid {
                    price: dec!(0.0020),
                    quantity: 1230
                },
                Bid {
                    price: dec!(0.0019),
                    quantity: 12
                }
            ]
        );

        assert_eq!(
            best_asks,
            vec![
                Ask {
                    price: dec!(0.0027),
                    quantity: 4
                },
                Ask {
                    price: dec!(0.0028),
                    quantity: 51
                },
                Ask {
                    price: dec!(0.0029),
                    quantity: 20
                },
                Ask {
                    price: dec!(0.0033),
                    quantity: 44
                },
                Ask {
                    price: dec!(0.0039),
                    quantity: 9
                }
            ]
        );
    }
}
