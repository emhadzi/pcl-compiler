fn next_token(src: &str, ind: usize) -> (Token, usize) {
    // As many as the types defined in enum
    let mut token_match = [true; 10];
    let mut cur = ind;
    let (mut last_match_mult, mut last_matching_type) = (true, TokenKind::Keyword);
    loop {
        // Check how token_match changes by processing src[cur]
        // TODO
        // If everything became invalid we match until this point
        if !token_match.contains(&true) {
            if last_match_mult {
                // handle ties based on priority
                return Err(());
            }
            let token_type = last_matching_type;
            return ();
        }

        cur += 1;
    }
}
