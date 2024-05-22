pub struct ProgressBar {
    i: usize,
    len: usize,
    pub bar: Vec<u8>
}
impl ProgressBar {
    const PREFIX_LEN: usize = 6;
    const POSTFIX_LEN: usize = 1;
    pub fn new(len: usize) -> ProgressBar {

        // making sure that len is an even number, then divide it by 2
        let l = ( if len & 1 == 1 { len+1 } else { len } ) >> 1;
 
        let mut bar: Vec<u8> = Vec::with_capacity( Self::PREFIX_LEN + l + Self::POSTFIX_LEN );
        
        let num_strting = format!("{: >4} [", 0_usize);
        bar.extend_from_slice(num_strting.as_bytes());
        // bar.push(b'[');
        for _ in 0..l { bar.push(b' ') };
        bar.push(b']');
        Self{i: 0, len, bar}

    }
    pub fn set_progress(&mut self, i: usize) {

        self.i = if i > 0 { i-1 } else { 0 };
        let num_strting = format!("{: >4} [", self.i);
        self.bar[..Self::PREFIX_LEN].clone_from_slice(num_strting.as_bytes());
        
        let transformed_i = self.i >> 1;
        let transformed_len = self.len >> 1;

        for j in Self::PREFIX_LEN..(Self::PREFIX_LEN + transformed_i) { self.bar[j] = b'=' };
        if self.i & 1 == 1 {self.bar[Self::PREFIX_LEN + transformed_i + 1] = b'-'};

        for j in (Self::PREFIX_LEN+transformed_i)..(Self::PREFIX_LEN+transformed_len) { self.bar[j] = b' ' };
        if self.len & 1 == 1 {
            self.bar[Self::PREFIX_LEN + transformed_len] = b' ';
            self.bar[Self::PREFIX_LEN + transformed_len + 1] = b']';
        } else {
            self.bar[Self::PREFIX_LEN+transformed_len] = b']';
        }

    }
    pub fn get_progress(&self) -> usize {
        self.i
    }

}
impl Iterator for ProgressBar {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i<self.len {

            let num_strting = format!("{: >4} [", self.i+1);

            self.bar[0..Self::PREFIX_LEN].clone_from_slice(num_strting.as_bytes());

            self.bar[Self::PREFIX_LEN + (self.i >> 1) ] 
                = if self.i & 1 == 1 { b'=' } else { b'-' };

            self.i += 1;
            Some(self.bar.clone())
        } else {
            None
        }
    }
}

const PREFIX_LEN: usize = 10;
const POSTFIX_LEN: usize = 1;
pub fn create_progress_bar(length: usize, progress: usize) -> Vec<u8> {


    let half_length = length >> 1;
    let length_is_odd = length & 1;

    let half_progress = progress >> 1;
    let progress_is_odd = progress & 1;

    let mut bar: Vec::<u8> = Vec::with_capacity(PREFIX_LEN + half_length + length_is_odd + POSTFIX_LEN);

    let s1 = format!("{}/{}", progress, length);

    let s2 = format!("{: >8} [", s1);
    bar.extend_from_slice(s2.as_bytes());

    for _ in 0..half_progress { bar.push(b'=') };
    if progress_is_odd == 1 { bar.push(b'-') }

    let remaining = half_length + length_is_odd - half_progress - progress_is_odd;
    for _ in 0..remaining { bar.push(b' ') };

    bar.push(b']');
    
    bar

}

