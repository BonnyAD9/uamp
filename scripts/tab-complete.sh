_uamp() {
    COMREPLY=( )
    while IFS= read -r line; do
        COMREPLY+=( "$line" )
    done < <( "$uamp" internal tab-complete ${COMP_CWORD} ${COMP_WORDS[*]} )
}
complete -F _uamp uamp
