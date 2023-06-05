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
        expected:" invalid argument",
    }
}

static_error_tests! {}
