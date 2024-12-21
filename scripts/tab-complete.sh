_uamp() {
    COMREPLY=( );
    "$__uamp_path" internal tab-complete ${COMP_CWORD} ${COMP_WORDS[*]} |
    while IFS= read -r line; do
        COMREPLY+=( "$line" );
        echo "$line";
    done
};
complete -F _uamp uamp
