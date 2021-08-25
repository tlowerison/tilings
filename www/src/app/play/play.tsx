import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import styles from "./styles.module.scss";
import { Autocomplete } from "@material-ui/lab";
import { Canvas, clearCanvas } from "components";
import { CircularProgress, TextField, Typography } from "@material-ui/core";
import { debounce, newGlobalId } from "utils";
import { getTiling, insertTileByPoint, removeTiling, setTiling, tilingSearch } from "client";
import { useHistory, useLocation } from "react-router";

type TextSearchItem = {
  id: number;
  title: string;
  labels: { content: string }[];
};

const defaultTextSearchItem = {
  id: -1,
  title: "",
  labels: [],
};

export const Play = () => {
  const history = useHistory();
  const location = useLocation();

  const globalId = useMemo(newGlobalId, []);
  const tilingId = useMemo(
    () => {
      if (!location.hash.startsWith("#")) {
        return null;
      }
      const tilingId = parseInt(location.hash.slice(1));
      return Number.isNaN(tilingId) ? null : tilingId;
    },
    [location.hash],
  );

  const [value, setValue] = useState<TextSearchItem>(defaultTextSearchItem);

  const containerRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  const [searchOptions, setSearchOptions] = useState<TextSearchItem[]>([]);
  const [searchLoading, setSearchLoading] = useState(false);

  const onInputChange = useMemo(
    () => debounce(async (event: Event) => {
      // @ts-ignore
      const inputValue = event?.target.value;
      if (typeof inputValue === "string") {
        if (inputValue === "") {
          setSearchOptions([]);
        } else {
          try {
            setSearchLoading(true);
            setSearchOptions(await tilingSearch(inputValue));
          } finally {
            setSearchLoading(false);
          }
        }
      }
    }, 100),
    [setSearchLoading, setSearchOptions],
  );

  const onChange = useCallback(
    (_: Event, textSearchItem: TextSearchItem | undefined) => {
      history.push(`${location.pathname}${!textSearchItem ? "" : `#${textSearchItem.id}`}`);
    },
    [history, location, searchOptions, setValue],
  );

  useEffect(
    () => {
      clearCanvas(globalId);
      if (tilingId) {
        setTiling(globalId, tilingId);
        const search = searchRef.current;
        if (search && !search.value) {
          (async () => {
            try {
              const { tiling: { id, title }, labels } = await getTiling(tilingId);
              setValue({ id, title, labels });
            } catch (e) {
              console.log(e);
            }
          })();
        } else {
          const option = searchOptions.find(({ id }) => id === tilingId);
          setValue(option || defaultTextSearchItem);
        }
      } else {
        removeTiling(globalId);
      }
    },
    [globalId, searchRef, setValue, tilingId],
  );

  // uncomment if using full screen canvas
  // useEffect(
  //   () => {
  //     const container = containerRef.current;
  //     const search = searchRef.current;
  //     if (container && search) {
  //       const { y } = container.getBoundingClientRect();
  //       search.style.marginBottom = `${-y}px`;
  //     }
  //   },
  //   [containerRef, searchRef],
  // );

  useEffect(() => {
    const html = Array.from(document.getElementsByTagName("html"))[0];
    html.scrollTop = 0;
    return () => removeTiling(globalId);
  }, []);

  return (
    <div className={styles.container}>
      <div className={styles.title}>
        <Typography variant="h3">Tilings</Typography>
      </div>
      <Autocomplete
        ref={searchRef}
        className={styles.search}
        getOptionLabel={
          ({ title, labels }) => `${
            title
          }${
            labels.length === 0
              ? ""
              : ` - ${labels.map(({ content }) => content).join(", ")}`
          }`
        }
        getOptionSelected={(option: TextSearchItem, value: TextSearchItem) => option.id === value.id}
        loading={searchLoading}
        {...{} /* @ts-ignore */}
        onChange={onChange}
        onInputChange={onInputChange}
        options={searchOptions}
        renderInput={(params) => (
          <TextField
            {...params}
            label="Search"
            variant="outlined"
            InputProps={{
              ...params.InputProps,
              endAdornment: (
                <React.Fragment>
                  {searchLoading ? <CircularProgress color="inherit" size={20} /> : null}
                  {params.InputProps.endAdornment}
                </React.Fragment>
              ),
            }}
          />
        )}
        value={value as any}
      />
      <Canvas
        containerRef={containerRef}
        globalId={globalId}
        height={window.outerHeight}
        onMouseMove={insertTileByPoint}
        onTouchMove={insertTileByPoint}
        scrollable
        width={window.outerWidth}
      />
    </div>
  );
};
