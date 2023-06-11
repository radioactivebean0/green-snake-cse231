rm -f line_counts.txt
touch line_counts.txt
make clean

for i in tests/*.snek; do
	cargo run -- $i ${i%.*}.s
    before=`wc -l < ${i%.*}.s`
    cargo run -- $i ${i%.*}.s --opt
	after=`wc -l < ${i%.*}.s`
    echo "$i $before --> $after" >> line_counts.txt
done
# for i in *.s; do
#     [ -f "$i" ] || break
#     wc -l "$i" >> line_counts.txt
# done
