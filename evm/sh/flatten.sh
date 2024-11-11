rm -rf flattened
mkdir flattened
for fileName in \
src/Endpoint.sol
  do
	echo $fileName
	flattened=flattened/`basename $fileName`
	forge flatten --output $flattened $fileName
done
