use std::cmp::Ordering;

use crate::SortOrder;

use rayon;

const PARALLEL_THRESHOLD: usize = 4096;

#[derive(Debug, PartialEq)]
struct Student {
    first_name: String,
    last_name: String,
    age: u8,
}

pub fn sort_by<T, F>(x: &mut [T], comparator: &F) -> Result<(), String>
where
    T: Send,
    F: Sync + Fn(&T, &T) -> Ordering,
{
    if x.len().is_power_of_two() {
        do_sort(x, true, comparator);
        Ok(())
    } else {
        Err(format!(
            "The length of x is not a power of two. (x.len(): {})",
            x.len(),
        ))
    }
}
pub fn sort<T: Ord>(x: &mut [T], order: &SortOrder) -> Result<(), String>
where
    T: Send,
{
    match order {
        &SortOrder::Ascending => sort_by(x, &|a, b| a.cmp(b)),
        &SortOrder::Descending => sort_by(x, &|a, b| b.cmp(a)),
    }
}

fn do_sort<T, F>(x: &mut [T], forward: bool, comparator: &F)
where
    T: Send,
    F: Sync + Fn(&T, &T) -> Ordering,
{
    if x.len() > 1 {
        let mid_point = x.len() / 2;
        let (first, second) = x.split_at_mut(mid_point);
        if mid_point > PARALLEL_THRESHOLD {
            rayon::join(
                || do_sort(first, true, comparator),
                || do_sort(second, false, comparator),
            );
        } else {
            do_sort(first, true, comparator);
            do_sort(second, false, comparator);
        }
        sub_sort(x, forward, comparator);
    }
}

fn sub_sort<T, F>(x: &mut [T], forward: bool, comparator: &F)
where
    F: Sync + Fn(&T, &T) -> Ordering,
    T: Send,
{
    if x.len() > 1 {
        compare_and_swap(x, forward, comparator);
        let mid_point = x.len() / 2;
        // 可変参照で分割
        let (first, second) = x.split_at_mut(mid_point);
        if mid_point > PARALLEL_THRESHOLD {
            rayon::join(
                || sub_sort(first, forward, comparator),
                || sub_sort(second, forward, comparator),
            );
        } else {
            sub_sort(first, forward, comparator);
            sub_sort(second, forward, comparator);
        }
    }
}

fn compare_and_swap<T, F>(x: &mut [T], forward: bool, comparator: &F)
where
    F: Fn(&T, &T) -> Ordering,
{
    let swap_condition = if forward {
        Ordering::Greater
    } else {
        Ordering::Less
    };
    let mid_point = x.len() / 2;
    for i in 0..mid_point {
        if comparator(&x[i], &x[mid_point + i]) == swap_condition {
            x.swap(i, mid_point + i);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::utils::{is_sorted_ascending, is_sorted_descending, new_u32_vec};
    use crate::SortOrder;

    use super::Student;
    use super::{sort, sort_by};

    impl Student {
        fn new(first_name: &str, last_name: &str, age: u8) -> Self {
            Self {
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
                age: age,
            }
        }
    }

    #[test]
    fn sort_students_by_ascending() {
        let taro = Student::new("Taro", "Yamada", 16);
        let hanako = Student::new("Hanako", "Yamada", 14);
        let kyoko = Student::new("Kyoko", "Ito", 15);
        let ryosuke = Student::new("Ryosuke", "Hayashi", 17);

        let mut x = vec![&taro, &hanako, &kyoko, &ryosuke];
        let expected = vec![&hanako, &kyoko, &taro, &ryosuke];

        assert_eq!(sort_by(&mut x, &|a, b| a.age.cmp(&b.age)), Ok(()));
        assert_eq!(x, expected);
    }

    #[test]
    fn sort_u32_large() {
        {
            let mut x = new_u32_vec(65536);
            assert_eq!(sort(&mut x, &SortOrder::Ascending), Ok(()));
            assert!(is_sorted_ascending(&x));
        }
        {
            let mut x = new_u32_vec(65536);
            assert_eq!(sort(&mut x, &SortOrder::Descending), Ok(()));
            assert!(is_sorted_descending(&x));
        }
    }
}
