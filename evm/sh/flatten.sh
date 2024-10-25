rm -rf flattened
mkdir flattened
for fileName in \
src/Router.sol
  do
	echo $fileName
	flattened=flattened/`basename $fileName`
	forge flatten --output $flattened $fileName
done
