
#for i in {0..180}; 
for i in $(seq -f "%03g" 0 2 359)
    do  target/release/healpix_plotter --min=-1 --max=1 --cmap planck --input-coord G --output-coord E --rotate-to $i,0 --fits cosmoglobe_DIRBE_06_I_n00512_DR2.fits --out out_$i.png --width 1600;
done
