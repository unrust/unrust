#!/bin/bash
for f in *.png
do
	convert -format dds -define dds:compression=DXT1 $f ${f%.*}.dds
done
