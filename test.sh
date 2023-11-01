./target/release/subimg -p './img/yellow_flower_forest.png' -O './img/forest.png' --all 2>/dev/null
if [ $? -ne 0 ]; then
  echo "pass"
else
  echo "error"
fi
./target/release/subimg './img/yellow_flower_forest.png' -O './img/forest.png' --all 2>/dev/null
if [ $? -ne 0 ]; then
  echo "pass"
else
  echo "pass"
fi

