#!/bin/sh
database="$1"
if [[ "$database" == "" ]]; then
  echo Missing database argument.
fi

credentials=$(credential-1password get --mode="mysql-$database" | grep -v '^#')
export $(echo "$credentials" | xargs -0)
goose ${@:2}
unset $(echo "$credentials" | sed -E 's/(.*)=.*/\1/' | xargs)
