cd src
sizes=(75 85 100 130 160 220 300 350 400 450)
#sizes=(75 80 85 90 95 100 110 120 130 150 160 180 220 250 280 300 310 330 350 370 380 400 420 450)
textBefore=$(less main.rs|head -23)
textAfter=$(less main.rs|tail -40)
before=$(less main.rs | head -n24 | tail -n1 | cut -c1-27)
after=$(less main.rs | head -n24 | tail -n1 | cut -c31-40)


for i in "${sizes[@]}"
do
	echo "$textBefore$before $i$after"> main.rs
	echo "$textAfter">> main.rs
	cd ..
	#echo " ************* doing for size $i *****************" >> output.txt	
	cargo run --release --features gcbench >> output.txt
	#cut c-86-93
	#echo "***************** ended for $i *****************" >> output.txt
	cd src
	echo $output >> output.txt
	
done
cd src

#cargo run --release --features features >> output.txt
