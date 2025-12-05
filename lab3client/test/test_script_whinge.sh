script_files=()

script_files+=("partial_hamlet_act_ii_script.txt")
script_files+=("test1_simple_valid.txt")
script_files+=("test2_no_title.txt")
script_files+=("test3_empty_script.txt")
script_files+=("test4_duplicate_lines.txt")
script_files+=("test5_missing_lines.txt")
script_files+=("test6_invalid_line_numbers.txt")
script_files+=("test7_config_issues.txt")
script_files+=("test8_scene_no_title.txt")
script_files+=("test9_extra_tokens.txt")
script_files+=("test10_empty_config.txt")
script_files+=("test11_three_scenes.txt")
script_files+=("test12_out_of_order.txt")
script_files+=("test13_no_lines.txt")

start=${1:-0}
end=${2:-13}

for ((i=start; i<=end; i++)); do
    cd "test_$i"
    cargo run "${script_files[$i]}" whinge
    cd ..
done
