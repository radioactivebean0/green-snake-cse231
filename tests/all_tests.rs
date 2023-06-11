mod infra;

// Your tests go here!
success_tests! {
    {
        name: make_vec_succ,
        file: "make_vec.snek",
        input: "5",
        expected: "[0, 0, 0, 0, 0]",
    },
    {
        name: vec_succ,
        file: "vec.snek",
        expected: "[0, 1, 2, 3]",
    },
    {
        name: vec_get_succ,
        file: "vec_get.snek",
        input: "3",
        expected: "3",
    },
    {
        name: linked_list_manipulations,
        file: "linked_list_manipulations.snek",
        expected: "1\n2\n3\n4\n5\n5\n4\n3\n2\n1\nnil"
    },
    {
        name: range_forced_gc,
        file: "range.snek",
        input: "5",
        heap_size: 25,
        expected: "[1, [2, [3, [4, [5, nil]]]]]"
    },
    {
        name: fact,
        file: "fact.snek",
        input: "10",
        expected: "3628800",
    },
    {
        name: even_odd_1,
        file: "even_odd.snek",
        input: "10",
        expected: "10\ntrue\ntrue",
    },
    {
        name: even_odd_2,
        file: "even_odd.snek",
        input: "9",
        expected: "9\nfalse\nfalse",
    },
    {
        name: add_lets,
        file: "add_lets.snek",
        expected: "30",
    },
    {
        name: add,
        file: "add.snek",
        expected: "15",
    },
    {
        name: add1,
        file: "add1.snek",
        expected: "73",
    },
    {
        name: binding,
        file: "binding.snek",
        expected: "5",
    },
    {
        name: chain_bindings,
        file: "chain_bindings.snek",
        expected: "65536",
    },
    {
        name: fac,
        file: "fac.snek",
        input: "5",
        expected: "120",
    },
    {
        name: false_val,
        file: "false_val.snek",
        expected: "false",
    },
    {
        name: if_expr,
        file: "if_expr.snek",
        expected: "-8",
    },
    {
        name: input_compare_1,
        file: "input_compare.snek",
        input: "2",
        expected: "false",
    },
    {
        name: input_compare_2,
        file: "input_compare.snek",
        input: "10",
        expected: "true",
    },
    {
        name: many_binding,
        file: "many_binding.snek",
        expected: "11",
    },
    {
        name: many_binding2,
        file: "many_binding2.snek",
        expected: "-90",
    },
    {
        name: nested_arith,
        file: "nested_arith.snek",
        expected: "25",
    },
    {
        name: nested_arith2,
        file: "nested_arith2.snek",
        expected: "50",
    },
    {
        name: nested_arith3,
        file: "nested_arith3.snek",
        expected: "1793",
    },
    {
        name: nested_binding,
        file: "nested_binding.snek",
        expected: "11",
    },
    {
        name: nested_reused_binding,
        file: "nested_reused_binding.snek",
        expected: "15",
    },
    {
        name: nested_lets,
        file: "nested_lets.snek",
        expected: "50",
    },
    {
        name: shadow_binding,
        file: "shadow_binding.snek",
        expected: "11",
    },
    {
        name: bool_eq,
        file: "bool_eq.snek",
        expected: "true",
    },
    {
        name: func_args_0,
        file: "many_args_funcs.snek",
        input: "0",
        expected: "true",
    },
    {
        name: func_args_1,
        file: "many_args_funcs.snek",
        input: "1",
        expected: "3",
    },
    {
        name: func_args_2,
        file: "many_args_funcs.snek",
        input: "2",
        expected: "5",
    },
    {
        name: func_args_3,
        file: "many_args_funcs.snek",
        input: "3",
        expected: "4",
    },
    {
        name: func_args_4,
        file: "many_args_funcs.snek",
        input: "4",
        expected: "1",
    },
    {
        name: func_args_5,
        file: "many_args_funcs.snek",
        input: "5",
        expected: "2",
    },
    {
        name: func_args_6,
        file: "many_args_funcs.snek",
        input: "6",
        expected: "4",
    },
    {
        name: func_args_7,
        file: "many_args_funcs.snek",
        input: "7",
        expected: "29",
    },
    {
        name: many_calls_1,
        file: "many_calls_1.snek",
        expected: "8",
    },
    {
        name: many_calls_2,
        file: "many_calls_2.snek",
        expected: "14",
    },
    {
        name: many_calls_3,
        file: "many_calls_3.snek",
        expected: "6"
    },
    {
        name: nested_prints,
        file: "nested_prints.snek",
        expected: "20\n1\n1\n50\n50"
    },
    {
        name: print_funcs,
        file: "print_funcs.snek",
        expected: "2\n2\n4\n10\n12\n14\n14"
    },
    {
        name: basic_input_1,
        file: "basic_input.snek",
        input: "2",
        expected: "2",
    },
    {
        name: basic_input_2,
        file: "basic_input.snek",
        input: "true",
        expected: "true",
    },
    {
        name: isnum_1,
        file: "isnum.snek",
        input: "2",
        expected: "true",
    },
    {
        name: isnum_2,
        file: "isnum.snek",
        input: "true",
        expected: "false",
    },
    {
        name: isbool_1,
        file: "isbool.snek",
        input: "2",
        expected: "false",
    },
    {
        name: isbool_2,
        file: "isbool.snek",
        input: "false",
        expected: "true",
    },
    {
        name: leq_1,
        file: "leq.snek",
        input: "9",
        expected: "false",
    },
    {
        name: leq_2,
        file: "leq.snek",
        input: "11",
        expected: "true",
    },
    {
        name: geq_1,
        file: "geq.snek",
        input: "150",
        expected: "true",
    },
    {
        name: geq_2,
        file: "geq.snek",
        input: "4",
        expected: "false",
    },
    {
        name: heap_1,
        file: "heap_1.snek",
        expected: "2",
    },
    {
        name: heap_2,
        file: "heap_2.snek",
        input: "0",
        expected: "3\n[2, 2, 2]"
    },
    {
        name: points_2,
        file: "points_2.snek",
        expected: "[3, 5]",
    },
    {
        name: heap_3,
        file: "heap_3.snek",
        expected: "[7, 6]",
    },
    {
        name: heap_ex1,
        file: "simple_heapex.snek",
        input: "1",
        expected: "2",
    },
    {
        name: heap_ex2,
        file: "simple_heapex.snek",
        input: "2",
        expected: "[1, 2, 3]\n3\n[2, 2, 2]",
    },
    {
        name: heap_ex3,
        file: "simple_heapex.snek",
        input: "3",
        expected: "[7, 6]",
    },
    {
        name: boa_points1,
        file: "points.boa.snek",
        input: "1",
        expected: "[3, 5]",
    },
    {
        name: boa_points2,
        file: "points.boa.snek",
        input: "2",
        expected: "[4, 4]",
    },
    {
        name: boa_points3,
        file: "points.boa.snek",
        input: "3",
        expected: "[1, 1]\n[2, 2]\n[4, 4]",
    },
    {
        name: boa_points4,
        file: "points.boa.snek",
        input: "4",
        expected: "[[2, 2], [4, 4]]",
    },
    {
        name: bst_1,
        file: "bst.boa.snek",
        input: "1",
        expected: "[false, false, false]",
    },
    {
        name: bst_2,
        file: "bst.boa.snek",
        input: "2",
        expected: "[1, false, [2, false, [3, false, [4, false, false]]]]",
    },
    {
        name: bst_3,
        file: "bst.boa.snek",
        input: "3",
        expected: "[4, [2, [1, false, false], [3, false, false]], [6, [5, false, false], [7, false, false]]]",
    },
    {
        name: bst_4,
        file: "bst.boa.snek",
        input: "4",
        expected: "[4, [3, [1, false, [2, false, false]], false], [6, false, false]]\nfalse\ntrue\ntrue",
    },
    {
        name: bst_5,
        file: "bst.boa.snek",
        input: "5",
        expected: "[false, false, false]\n[4, false, false]\n[4, [3, false, false], false]\n[4, [3, [1, false, false], false], false]\n[4, [3, [1, false, [2, false, false]], false], false]\n[4, [3, [1, false, [2, false, false]], false], [6, false, false]]",
    },
    {
        name: bst_6,
        file: "bst.boa.snek",
        input: "6",
        expected: "[8, [6, [5, false, false], [7, false, false]], [10, [9, false, false], [11, false, false]]]\n[8, [6, [5, [4, false, false], false], [7, false, false]], [10, [9, false, false], [11, false, false]]]",
    }
}

runtime_error_tests! {
    {
        name: make_vec_oom,
        file: "make_vec.snek",
        input: "5",
        heap_size: 5,
        expected: "out of memory",
    },
    {
        name: vec_get_oob,
        file: "vec_get.snek",
        input: "5",
        expected: "",
    },
    {
        name: add_overflow,
        file: "add_overflow.snek",
        expected: "overflow",
    },
    {
        name: mul_overflow,
        file: "mul_overflow.snek",
        expected: "overflow",
    },
    {
        name: invalid_argument_1,
        file: "invalid_argument_1.snek",
        expected: "invalid argument",
    },
    {
        name: invalid_argument_2,
        file: "invalid_argument_2.snek",
        expected: "invalid argument",
    },
    {
        name: invalid_argument_3,
        file: "invalid_argument_3.snek",
        expected: "invalid argument",
    },
    {
        name: invalid_argument_4,
        file: "invalid_argument_4.snek",
        expected: "invalid argument",
    },
    {
        name: input_compare_3,
        file: "input_compare.snek",
        input: "true",
        expected: "invalid argument",
    },
    {
        name: invalid_input_1,
        file: "invalid_input.snek",
        input: "asdfa",
        expected: "ParseIntError",
    },
    {
        name: invalid_input_2,
        file: "invalid_input.snek",
        input: "1.23",
        expected: "ParseIntError",
    },
    {
        name: invalid_input_3,
        file: "invalid_input.snek",
        input: "46116860184273879045496581",
        expected: "ParseIntError",
    },
    {
        name: heap_out_of_bounds_1,
        file: "heap_2.snek",
        input: "1",
        expected: "bounds",
    },
    {
        name: heap_out_of_bounds_2,
        file: "heap_6.snek",
        expected: "bounds",
    },
    {
        name: not_vek,
        file: "not_vek.snek",
        input: "2",
        expected: "invalid argument",
    },
    {
        name: vek_bool,
        file: "vek_bool.snek",
        expected: "invalid argument",
    },
    {
        name: heap_4,
        file: "heap_4.snek",
        input: "true",
        expected: "invalid argument",
    },
    {
        name: heap_5,
        file: "heap_5.snek",
        expected:"invalid argument",
    },
    {
        name: error_heap_tag1,
        file: "error-tag.boa.snek",
        input: "1",
        expected: "invalid argument",
    },
    {
        name: error_heap_tag2,
        file: "error-tag.boa.snek",
        input: "2",
        expected: "invalid argument",
    },
    {
        name: error_bounds1,
        file: "error-bounds.boa.snek",
        input: "1",
        expected: "bounds",
    },
    {
        name: error_bounds2,
        file: "error-bounds.boa.snek",
        input: "2",
        expected: "bounds",
    }
}

static_error_tests! {}

profile_tests! {
    {
        name: profile_make_vec_succ,
        file: "make_vec.snek",
        input: "5",
        expected: "[0, 0, 0, 0, 0]",
    },
    {
        name: profile_vec_succ,
        file: "vec.snek",
        expected: "[0, 1, 2, 3]",
    },
    {
        name: profile_vec_get_succ,
        file: "vec_get.snek",
        input: "3",
        expected: "3",
    },
    {
        name: profile_linked_list_manipulations,
        file: "linked_list_manipulations.snek",
        expected: "1\n2\n3\n4\n5\n5\n4\n3\n2\n1\nnil"
    },
    {
        name: profile_range_forced_gc,
        file: "range.snek",
        input: "5",
        heap_size: 25,
        expected: "[1, [2, [3, [4, [5, nil]]]]]"
    },
    {
        name: profile_fact,
        file: "fact.snek",
        input: "10",
        expected: "3628800",
    },
    {
        name: profile_even_odd_1,
        file: "even_odd.snek",
        input: "10",
        expected: "10\ntrue\ntrue",
    },
    {
        name: profile_even_odd_2,
        file: "even_odd.snek",
        input: "9",
        expected: "9\nfalse\nfalse",
    },
    {
        name: profile_add_lets,
        file: "add_lets.snek",
        expected: "30",
    },
    {
        name: profile_add,
        file: "add.snek",
        expected: "15",
    },
    {
        name: profile_add1,
        file: "add1.snek",
        expected: "73",
    },
    {
        name: profile_binding,
        file: "binding.snek",
        expected: "5",
    },
    {
        name: profile_chain_bindings,
        file: "chain_bindings.snek",
        expected: "65536",
    },
    {
        name: profile_fac,
        file: "fac.snek",
        input: "5",
        expected: "120",
    },
    {
        name: profile_false_val,
        file: "false_val.snek",
        expected: "false",
    },
    {
        name: profile_if_expr,
        file: "if_expr.snek",
        expected: "-8",
    },
    {
        name: profile_input_compare_1,
        file: "input_compare.snek",
        input: "2",
        expected: "false",
    },
    {
        name: profile_input_compare_2,
        file: "input_compare.snek",
        input: "10",
        expected: "true",
    },
    {
        name: profile_many_binding,
        file: "many_binding.snek",
        expected: "11",
    },
    {
        name: profile_many_binding2,
        file: "many_binding2.snek",
        expected: "-90",
    },
    {
        name: profile_nested_arith,
        file: "nested_arith.snek",
        expected: "25",
    },
    {
        name: profile_nested_arith2,
        file: "nested_arith2.snek",
        expected: "50",
    },
    {
        name: profile_nested_arith3,
        file: "nested_arith3.snek",
        expected: "1793",
    },
    {
        name: profile_nested_binding,
        file: "nested_binding.snek",
        expected: "11",
    },
    {
        name: profile_nested_reused_binding,
        file: "nested_reused_binding.snek",
        expected: "15",
    },
    {
        name: profile_nested_lets,
        file: "nested_lets.snek",
        expected: "50",
    },
    {
        name: profile_shadow_binding,
        file: "shadow_binding.snek",
        expected: "11",
    },
    {
        name: profile_bool_eq,
        file: "bool_eq.snek",
        expected: "true",
    },
    {
        name: profile_func_args_0,
        file: "many_args_funcs.snek",
        input: "0",
        expected: "true",
    },
    {
        name: profile_func_args_1,
        file: "many_args_funcs.snek",
        input: "1",
        expected: "3",
    },
    {
        name: profile_func_args_2,
        file: "many_args_funcs.snek",
        input: "2",
        expected: "5",
    },
    {
        name: profile_func_args_3,
        file: "many_args_funcs.snek",
        input: "3",
        expected: "4",
    },
    {
        name: profile_func_args_4,
        file: "many_args_funcs.snek",
        input: "4",
        expected: "1",
    },
    {
        name: profile_func_args_5,
        file: "many_args_funcs.snek",
        input: "5",
        expected: "2",
    },
    {
        name: profile_func_args_6,
        file: "many_args_funcs.snek",
        input: "6",
        expected: "4",
    },
    {
        name: profile_func_args_7,
        file: "many_args_funcs.snek",
        input: "7",
        expected: "29",
    },
    {
        name: profile_many_calls_1,
        file: "many_calls_1.snek",
        expected: "8",
    },
    {
        name: profile_many_calls_2,
        file: "many_calls_2.snek",
        expected: "14",
    },
    {
        name: profile_many_calls_3,
        file: "many_calls_3.snek",
        expected: "6"
    },
    {
        name: profile_nested_prints,
        file: "nested_prints.snek",
        expected: "20\n1\n1\n50\n50"
    },
    {
        name: profile_print_funcs,
        file: "print_funcs.snek",
        expected: "2\n2\n4\n10\n12\n14\n14"
    },
    {
        name: profile_basic_input_1,
        file: "basic_input.snek",
        input: "2",
        expected: "2",
    },
    {
        name: profile_basic_input_2,
        file: "basic_input.snek",
        input: "true",
        expected: "true",
    },
    {
        name: profile_isnum_1,
        file: "isnum.snek",
        input: "2",
        expected: "true",
    },
    {
        name: profile_isnum_2,
        file: "isnum.snek",
        input: "true",
        expected: "false",
    },
    {
        name: profile_isbool_1,
        file: "isbool.snek",
        input: "2",
        expected: "false",
    },
    {
        name: profile_isbool_2,
        file: "isbool.snek",
        input: "false",
        expected: "true",
    },
    {
        name: profile_leq_1,
        file: "leq.snek",
        input: "9",
        expected: "false",
    },
    {
        name: profile_leq_2,
        file: "leq.snek",
        input: "11",
        expected: "true",
    },
    {
        name: profile_geq_1,
        file: "geq.snek",
        input: "150",
        expected: "true",
    },
    {
        name: profile_geq_2,
        file: "geq.snek",
        input: "4",
        expected: "false",
    },
    {
        name: profile_heap_1,
        file: "heap_1.snek",
        expected: "2",
    },
    {
        name: profile_heap_2,
        file: "heap_2.snek",
        input: "0",
        expected: "3\n[2, 2, 2]"
    },
    {
        name: profile_points_2,
        file: "points_2.snek",
        expected: "[3, 5]",
    },
    {
        name: profile_heap_3,
        file: "heap_3.snek",
        expected: "[7, 6]",
    },
    {
        name: profile_heap_ex1,
        file: "simple_heapex.snek",
        input: "1",
        expected: "2",
    },
    {
        name: profile_heap_ex2,
        file: "simple_heapex.snek",
        input: "2",
        expected: "[1, 2, 3]\n3\n[2, 2, 2]",
    },
    {
        name: profile_heap_ex3,
        file: "simple_heapex.snek",
        input: "3",
        expected: "[7, 6]",
    },
    {
        name: profile_boa_points1,
        file: "points.boa.snek",
        input: "1",
        expected: "[3, 5]",
    },
    {
        name: profile_boa_points2,
        file: "points.boa.snek",
        input: "2",
        expected: "[4, 4]",
    },
    {
        name: profile_boa_points3,
        file: "points.boa.snek",
        input: "3",
        expected: "[1, 1]\n[2, 2]\n[4, 4]",
    },
    {
        name: profile_boa_points4,
        file: "points.boa.snek",
        input: "4",
        expected: "[[2, 2], [4, 4]]",
    },
    {
        name: profile_bst_1,
        file: "bst.boa.snek",
        input: "1",
        expected: "[false, false, false]",
    },
    {
        name: profile_bst_2,
        file: "bst.boa.snek",
        input: "2",
        expected: "[1, false, [2, false, [3, false, [4, false, false]]]]",
    },
    {
        name: profile_bst_3,
        file: "bst.boa.snek",
        input: "3",
        expected: "[4, [2, [1, false, false], [3, false, false]], [6, [5, false, false], [7, false, false]]]",
    },
    {
        name: profile_bst_4,
        file: "bst.boa.snek",
        input: "4",
        expected: "[4, [3, [1, false, [2, false, false]], false], [6, false, false]]\nfalse\ntrue\ntrue",
    },
    {
        name: profile_bst_5,
        file: "bst.boa.snek",
        input: "5",
        expected: "[false, false, false]\n[4, false, false]\n[4, [3, false, false], false]\n[4, [3, [1, false, false], false], false]\n[4, [3, [1, false, [2, false, false]], false], false]\n[4, [3, [1, false, [2, false, false]], false], [6, false, false]]",
    },
    {
        name: profile_bst_6,
        file: "bst.boa.snek",
        input: "6",
        expected: "[8, [6, [5, false, false], [7, false, false]], [10, [9, false, false], [11, false, false]]]\n[8, [6, [5, [4, false, false], false], [7, false, false]], [10, [9, false, false], [11, false, false]]]",
    },
    {
        name: profile_bigloop,
        file: "bigloop.snek",
        input: "100000000",
        expected: "100",
    }
}