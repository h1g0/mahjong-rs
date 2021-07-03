use std::cmp::*;

use crate::hand::Hand;
use crate::hand_info::winning_hand::WinningHandForm;
use crate::tile::Tile;

/// 向聴数などの手牌に関する情報を計算する
#[derive(Debug, Eq)]
pub struct HandAnalyzer {
    /// 向聴数：あと牌を何枚交換すれば聴牌できるかの最小数。聴牌状態が`0`、和了が`-1`。
    pub shanten: i32,
    pub form: WinningHandForm,
}
impl Ord for HandAnalyzer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.shanten.cmp(&other.shanten)
    }
}

impl PartialOrd for HandAnalyzer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HandAnalyzer {
    fn eq(&self, other: &Self) -> bool {
        self.shanten == other.shanten
    }
}

impl HandAnalyzer {
    /// 向聴数を計算する
    ///
    /// 七対子・国士無双・通常の3つの和了形に対してそれぞれ向聴数を求め、最小のものを返す。
    pub fn calc(hand: &Hand) -> HandAnalyzer {
        let sp = HandAnalyzer::calc_by_form(hand, WinningHandForm::SevenPairs);
        let to = HandAnalyzer::calc_by_form(hand, WinningHandForm::ThirteenOrphens);
        let normal = HandAnalyzer::calc_by_form(hand, WinningHandForm::Normal);
        return min(min(sp, to), normal);
    }

    /// 和了形を指定して向聴数を計算する
    /// # Examples
    ///
    /// ```
    /// use mahjong_rs::hand::*;
    /// use mahjong_rs::hand_info::hand_analyzer::*;
    /// use mahjong_rs::hand_info::winning_hand::*;
    ///
    /// // 国士無双で和了る
    /// let to_test_str = "19m19p19s1234567z 1m";
    /// let to_test = Hand::from(to_test_str);
    /// assert_eq!(
    ///   HandAnalyzer::calc_by_form(&to_test, WinningHandForm::ThirteenOrphens).shanten,
    ///   -1
    /// );
    ///
    /// // 七対子で和了る
    /// let sp_test_str = "1122m3344p5566s7z 7z";
    /// let sp_test = Hand::from(sp_test_str);
    /// assert_eq!(
    ///   HandAnalyzer::calc_by_form(&sp_test, WinningHandForm::SevenPairs).shanten,
    ///   -1
    /// );
    /// ```
    pub fn calc_by_form(hand: &Hand, form: WinningHandForm) -> HandAnalyzer {
        return match form {
            WinningHandForm::SevenPairs => HandAnalyzer {
                shanten: HandAnalyzer::calc_seven_pairs(hand),
                form: WinningHandForm::SevenPairs,
            },
            WinningHandForm::ThirteenOrphens => HandAnalyzer {
                shanten: HandAnalyzer::calc_thirteen_orphens(hand),
                form: WinningHandForm::ThirteenOrphens,
            },
            WinningHandForm::Normal => HandAnalyzer {
                shanten: HandAnalyzer::calc_normal_form(hand),
                form: WinningHandForm::Normal,
            },
        };
    }

    /// 七対子への向聴数を計算する
    fn calc_seven_pairs(hand: &Hand) -> i32 {
        let mut pair: u32 = 0;
        let mut kind: u32 = 0;
        let t = hand.summarize_tiles();

        for i in 0..Tile::LEN {
            if t[i] > 0 {
                kind += 1;
                if t[i] >= 2 {
                    pair += 1;
                }
            }
        }
        let num_to_win: i32 = (7 - pair + if kind < 7 { 7 - kind } else { 0 }) as i32;
        return num_to_win - 1;
    }

    /// 国士無双への向聴数を計算する
    fn calc_thirteen_orphens(hand: &Hand) -> i32 {
        let to_tiles = [
            Tile::M1,
            Tile::M9,
            Tile::P1,
            Tile::P9,
            Tile::S1,
            Tile::S9,
            Tile::Z1,
            Tile::Z2,
            Tile::Z3,
            Tile::Z4,
            Tile::Z5,
            Tile::Z6,
            Tile::Z7,
        ];
        let mut pair: u32 = 0;
        let mut kind: u32 = 0;
        let t = hand.summarize_tiles();

        for i in &to_tiles {
            if t[*i as usize] > 0 {
                kind = kind + 1;
                if t[*i as usize] >= 2 {
                    pair += 1;
                }
            }
        }
        let num_to_win: i32 = (14 - kind - if pair > 0 { 1 } else { 0 }) as i32;
        return num_to_win - 1;
    }

    /// 通常の役への向聴数を計算する
    fn calc_normal_form(hand: &Hand) -> i32 {
        let mut t = hand.summarize_tiles();
        let mut shanten = 100;

        let same3: u32 = 0;
        let sequential3: u32 = 0;
        let mut same2: u32 = 0;
        let sequential2: u32 = 0;

        // 先に独立した牌を抜き出しておく
        let independent_same3 = HandAnalyzer::count_independent_same_3(&mut t);
        let independent_sequential3 = HandAnalyzer::count_independent_sequential_3(&mut t);
        let independent_single = HandAnalyzer::count_independent_single(&mut t);

        // 雀頭を抜き出す
        for i in Tile::M1..=Tile::Z7 {
            if t[i as usize] >= 2 {
                same2 += 1;
                t[i as usize] -= 2;
                shanten = count_normal_shanten_by_recursive(
                    0,
                    independent_same3,
                    independent_sequential3,
                    same3,
                    sequential3,
                    same2,
                    sequential2,
                    &mut t,
                    shanten,
                );
                t[i as usize] += 2;
                same2 -= 1;
            }
        }

        // 雀頭がない場合
        shanten = count_normal_shanten_by_recursive(
            0,
            independent_same3,
            independent_sequential3,
            0,
            0,
            0,
            0,
            &mut t,
            shanten,
        );
        return shanten;
    }
    /// 独立した（順子になり得ない）刻子の数を返す
    fn count_independent_same_3(summarized_hand: &mut Vec<u32>) -> u32 {
        let mut result: u32 = 0;
        for i in Tile::M1..=Tile::Z7 {
            match i {
                Tile::M1 | Tile::P1 | Tile::S1 => {
                    if summarized_hand[i as usize] >= 3
                        && summarized_hand[i as usize + 1] == 0
                        && summarized_hand[i as usize + 2] == 0
                    {
                        summarized_hand[i as usize] -= 3;
                        result += 1;
                    }
                }
                Tile::M2 | Tile::P2 | Tile::S2 => {
                    if summarized_hand[i as usize - 1] == 0
                        && summarized_hand[i as usize] >= 3
                        && summarized_hand[i as usize + 1] == 0
                        && summarized_hand[i as usize + 2] == 0
                    {
                        summarized_hand[i as usize] -= 3;
                        result += 1;
                    }
                }
                Tile::M3..=Tile::M7 | Tile::P3..=Tile::P7 | Tile::S3..=Tile::S7 => {
                    if summarized_hand[i as usize - 2] == 0
                        && summarized_hand[i as usize - 1] == 0
                        && summarized_hand[i as usize] >= 3
                        && summarized_hand[i as usize + 1] == 0
                        && summarized_hand[i as usize + 2] == 0
                    {
                        summarized_hand[i as usize] -= 3;
                        result += 1;
                    }
                }
                Tile::M8 | Tile::P8 | Tile::S8 => {
                    if summarized_hand[i as usize - 2] == 0
                        && summarized_hand[i as usize - 1] == 0
                        && summarized_hand[i as usize] >= 3
                        && summarized_hand[i as usize + 1] == 0
                    {
                        summarized_hand[i as usize] -= 3;
                        result += 1;
                    }
                }
                Tile::M9 | Tile::P9 | Tile::S9 => {
                    if summarized_hand[i as usize - 2] == 0
                        && summarized_hand[i as usize - 1] == 0
                        && summarized_hand[i as usize] >= 3
                    {
                        summarized_hand[i as usize] -= 3;
                        result += 1;
                    }
                }
                Tile::Z1..=Tile::Z7 => {
                    if summarized_hand[i as usize] >= 3 {
                        summarized_hand[i as usize] -= 3;
                        result += 1;
                    }
                }
                _ => {
                    panic! {"unknown tile index!"}
                }
            }
        }
        return result;
    }

    /// 独立した（他の順子と複合し得ない）順子の数を返す
    /// i.e. xx567xxのような順子
    fn count_independent_sequential_3(summarized_hand: &mut Vec<u32>) -> u32 {
        let mut result: u32 = 0;
        // 先に一盃口の処理をしてから通常の処理
        for i in (1..=2).rev() {
            // 一萬、一筒、一索のインデックス位置
            for j in (Tile::M1..=Tile::S9).step_by(9) {
                // 一*～七*のインデックス位置
                for k in 0..=6 {
                    let l: usize = (j + k) as usize;
                    //三*以上のとき-2の牌が存在したらチェックしない
                    // i.e. チェック下限はxx345
                    if k >= 2 && summarized_hand[l - 2] > 0 {
                        continue;
                    }
                    //二*以上のとき-1の牌が存在したらチェックしない
                    // i.e. チェック下限はx234
                    if k >= 1 && summarized_hand[l - 1] > 0 {
                        continue;
                    }
                    //六*以下で+3の牌が存在したらチェックしない
                    // i.e. チェック上限は678x
                    if k <= 5 && summarized_hand[l + 3] > 0 {
                        continue;
                    }
                    //五*以下で+4の牌が存在したらチェックしない
                    // i.e. チェック上限は567xx
                    if k <= 4 && summarized_hand[l + 4] > 0 {
                        continue;
                    }
                    if summarized_hand[l] == i
                        && summarized_hand[l + 1] == i
                        && summarized_hand[l + 2] == i
                    {
                        summarized_hand[l] -= i;
                        summarized_hand[l + 1] -= i;
                        summarized_hand[l + 2] -= i;
                        result += i;
                    }
                }
            }
        }
        return result;
    }

    /// 独立した（他の順子や刻子などと複合し得ない）牌の数を返す
    fn count_independent_single(summarized_hand: &mut Vec<u32>) -> u32 {
        let mut result: u32 = 0;
        for i in Tile::M1..=Tile::Z7 {
            match i {
                Tile::M1 | Tile::P1 | Tile::S1 => {
                    if summarized_hand[i as usize] == 1
                        && summarized_hand[i as usize + 1] == 0
                        && summarized_hand[i as usize + 2] == 0
                    {
                        summarized_hand[i as usize] -= 1;
                        result += 1;
                    }
                }
                Tile::M2 | Tile::P2 | Tile::S2 => {
                    if summarized_hand[i as usize - 1] == 0
                        && summarized_hand[i as usize] == 1
                        && summarized_hand[i as usize + 1] == 0
                        && summarized_hand[i as usize + 2] == 0
                    {
                        summarized_hand[i as usize] -= 1;
                        result += 1;
                    }
                }
                Tile::M3..=Tile::M7 | Tile::P3..=Tile::P7 | Tile::S3..=Tile::S7 => {
                    if summarized_hand[i as usize - 2] == 0
                        && summarized_hand[i as usize - 1] == 0
                        && summarized_hand[i as usize] == 1
                        && summarized_hand[i as usize + 1] == 0
                        && summarized_hand[i as usize + 2] == 0
                    {
                        summarized_hand[i as usize] -= 1;
                        result += 1;
                    }
                }
                Tile::M8 | Tile::P8 | Tile::S8 => {
                    if summarized_hand[i as usize - 2] == 0
                        && summarized_hand[i as usize - 1] == 0
                        && summarized_hand[i as usize] == 1
                        && summarized_hand[i as usize + 1] == 0
                    {
                        summarized_hand[i as usize] -= 1;
                        result += 1;
                    }
                }
                Tile::M9 | Tile::P9 | Tile::S9 => {
                    if summarized_hand[i as usize - 2] == 0
                        && summarized_hand[i as usize - 1] == 0
                        && summarized_hand[i as usize] == 1
                    {
                        summarized_hand[i as usize] -= 1;
                        result += 1;
                    }
                }
                Tile::Z1..=Tile::Z7 => {
                    if summarized_hand[i as usize] == 1 {
                        summarized_hand[i as usize] -= 1;
                        result += 1;
                    }
                }
                _ => {
                    panic! {"unknown tile index!"}
                }
            }
        }
        return result;
    }
}

fn count_normal_shanten_by_recursive(
    idx: u32,
    independent_same3: u32,
    independent_sequential3: u32,
    same3: u32,
    sequential3: u32,
    same2: u32,
    sequential2: u32,
    summarized_hand: &mut Vec<u32>,
    mut shanten_min: i32,
) -> i32 {
    count_same_or_sequential_3(
        idx,
        independent_same3,
        independent_sequential3,
        same3,
        sequential3,
        same2,
        sequential2,
        summarized_hand,
        shanten_min,
    );
    count_2(
        idx,
        independent_same3,
        independent_sequential3,
        same3,
        sequential3,
        same2,
        sequential2,
        summarized_hand,
        shanten_min,
    );
    let shanten = calc_normal_shanten(
        independent_same3,
        independent_sequential3,
        same3,
        sequential3,
        same2,
        sequential2,
    );
    if shanten < shanten_min {
        shanten_min = shanten;
    }
    return shanten_min;
}

/// 面子（刻子および順子）の数を返す
/// # returns
/// (刻子, 順子)
fn count_same_or_sequential_3(
    idx: u32,
    independent_same3: u32,
    independent_sequential3: u32,
    mut same3: u32,
    mut sequential3: u32,
    same2: u32,
    sequential2: u32,
    summarized_hand: &mut Vec<u32>,
    shanten_min: i32,
) -> (u32, u32) {
    for i in idx..=Tile::Z7 {
        // 刻子カウント
        if summarized_hand[i as usize] >= 3 {
            //block3 += 1;
            same3 += 1;
            summarized_hand[i as usize] -= 3;
            count_normal_shanten_by_recursive(
                i,
                independent_same3,
                independent_sequential3,
                same3,
                sequential3,
                same2,
                sequential2,
                summarized_hand,
                shanten_min,
            );
            summarized_hand[i as usize] += 3;
            same3 -= 1;
        }

        //順子カウント
        if ((i >= Tile::M1 && i <= Tile::M7)
            || (i >= Tile::P1 && i <= Tile::P7)
            || (i >= Tile::S1 && i <= Tile::S7))
            && summarized_hand[i as usize] >= 1
            && summarized_hand[i as usize + 1] >= 1
            && summarized_hand[i as usize + 2] >= 1
        {
            //block3 += 1;
            sequential3 += 1;
            summarized_hand[i as usize] -= 1;
            summarized_hand[i as usize + 1] -= 1;
            summarized_hand[i as usize + 2] -= 1;
            count_normal_shanten_by_recursive(
                i,
                independent_same3,
                independent_sequential3,
                same3,
                sequential3,
                same2,
                sequential2,
                summarized_hand,
                shanten_min,
            );
            summarized_hand[i as usize] += 1;
            summarized_hand[i as usize + 1] += 1;
            summarized_hand[i as usize + 2] += 1;
            sequential3 -= 1;
        }
    }
    return (same3, sequential3);
}

/// 対子・塔子・嵌張カウント
/// # returns
/// (対子,塔子・嵌張)
fn count_2(
    idx: u32,
    independent_same3: u32,
    independent_sequential3: u32,
    same3: u32,
    sequential3: u32,
    mut same2: u32,
    mut sequential2: u32,
    summarized_hand: &mut Vec<u32>,
    shanten: i32,
) -> (u32, u32) {
    for i in idx..=Tile::Z7 {
        // 対子
        if summarized_hand[i as usize] == 2 {
            same2 += 1;
            summarized_hand[i as usize] -= 2;
            count_normal_shanten_by_recursive(
                idx,
                independent_same3,
                independent_sequential3,
                same3,
                sequential3,
                same2,
                sequential2,
                summarized_hand,
                shanten,
            );
            summarized_hand[i as usize] += 2;
            same2 -= 1;
        }
        //数牌
        if i <= Tile::S9 && (i >= Tile::M1 && i <= Tile::M7)
            || (i >= Tile::P1 && i <= Tile::P7)
            || (i >= Tile::S1 && i <= Tile::S7)
        {
            // 塔子
            if summarized_hand[i as usize] >= 1 && summarized_hand[i as usize + 1] >= 1 {
                sequential2 += 1;
                summarized_hand[i as usize] -= 1;
                summarized_hand[i as usize + 1] -= 1;
                count_normal_shanten_by_recursive(
                    idx,
                    independent_same3,
                    independent_sequential3,
                    same3,
                    sequential3,
                    same2,
                    sequential2,
                    summarized_hand,
                    shanten,
                );
                summarized_hand[i as usize] += 1;
                summarized_hand[i as usize + 1] += 1;
                sequential2 -= 1;
            }
            //嵌張
            if summarized_hand[i as usize] >= 1
                && summarized_hand[i as usize + 1] == 0
                && summarized_hand[i as usize + 2] >= 1
            {
                sequential2 += 1;
                summarized_hand[i as usize] -= 1;
                summarized_hand[i as usize + 2] -= 1;
                count_normal_shanten_by_recursive(
                    idx,
                    independent_same3,
                    independent_sequential3,
                    same3,
                    sequential3,
                    same2,
                    sequential2,
                    summarized_hand,
                    shanten,
                );
                summarized_hand[i as usize] += 1;
                summarized_hand[i as usize + 2] += 1;
                sequential2 -= 1;
            }
        }
    }
    return (same2, sequential2);
}

fn calc_normal_shanten(
    independent_same3: u32,
    independent_sequential3: u32,
    same3: u32,
    sequential3: u32,
    same2: u32,
    sequential2: u32,
) -> i32 {
    let block3 = independent_same3 + independent_sequential3 + same3 + sequential3;
    let block2 = same2 + sequential2;
    return 8 - (block3 * 2 + block2) as i32;
}

/// ユニットテスト
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// 七対子で和了った
    fn win_by_seven_pairs() {
        let test_str = "1122m3344p5566s1z 1z";
        let test = Hand::from(test_str);
        assert_eq!(
            HandAnalyzer::calc_by_form(&test, WinningHandForm::SevenPairs).shanten,
            -1
        );
    }

    #[test]
    /// 国士無双で和了った
    fn win_by_thirteen_orphens() {
        let test_str = "19m19p19s1234567z 1m";
        let test = Hand::from(test_str);
        assert_eq!(
            HandAnalyzer::calc_by_form(&test, WinningHandForm::ThirteenOrphens).shanten,
            -1
        );
    }

    #[test]
    /// 七対子を聴牌
    fn zero_shanten_to_seven_pairs() {
        let test_str = "226699m99p228s66z 1z";
        let test = Hand::from(test_str);
        assert_eq!(
            HandAnalyzer::calc_by_form(&test, WinningHandForm::SevenPairs).shanten,
            0
        );
    }
    #[test]
    /// 同じ牌が3枚ある状態で七対子を聴牌
    fn zero_shanten_to_seven_pairs_2() {
        let test_str = "226699m99p222s66z 1z";
        let test = Hand::from(test_str);
        assert_eq!(
            HandAnalyzer::calc_by_form(&test, WinningHandForm::SevenPairs).shanten,
            0
        );
    }
    #[test]
    /// 国士無双を聴牌
    fn zero_shanten_to_orphens() {
        let test_str = "19m19p11s1234567z 5m";
        let test = Hand::from(test_str);
        assert_eq!(
            HandAnalyzer::calc_by_form(&test, WinningHandForm::ThirteenOrphens).shanten,
            0
        );
    }

    #[test]
    /// 同じ牌が4枚ある状態で七対子は認められない（一向聴とみなす）
    fn seven_pairs_with_4_same_tiles() {
        let test_str = "1122m3344p5555s1z 1z";
        let test = Hand::from(test_str);
        assert_eq!(
            HandAnalyzer::calc_by_form(&test, WinningHandForm::SevenPairs).shanten,
            1
        );
    }

    #[test]
    /// 立直で和了った
    fn win_by_riichi() {
        let test_str = "123m456p789s1112z 2z";
        let test = Hand::from(test_str);
        assert_eq!(
            HandAnalyzer::calc_by_form(&test, WinningHandForm::Normal).shanten,
            -1
        );
    }
}
