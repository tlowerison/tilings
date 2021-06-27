import React, { useCallback, useMemo, useRef, useState } from "react";
import { Autocomplete } from "@material-ui/lab";
import { Canvas, clearCanvas } from "./canvas";
import { CircularProgress, TextField } from "@material-ui/core";
import { debounce } from "util/input";
import { handleEvent, setTiling, tilingSearch } from "client";
import styles from "./styles.module.scss";

const isMobile = () => window.outerWidth > 800;

const canvasWidth = isMobile()
  ? window.outerWidth - 100
  : window.outerWidth;
const canvasHeight = isMobile()
  ? window.outerHeight - 150
  : window.outerHeight - 120;

type TextSearchItem = {
  id: number;
  title: string;
  table: string;
  labels: { content: string }[];
};

export const Tilings = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const [searchOptions, setSearchOptions] = useState<TextSearchItem[]>([]);
  const [searchLoading, setSearchLoading] = useState(false);

  const onInputChange = useMemo(
    () => debounce(async (event: Event) => {
      // @ts-ignore
      const search = event.target.value;
      if (typeof search === "string") {
        if (search === "") {
          setSearchOptions([]);
        } else {
          try {
            setSearchLoading(true);
            setSearchOptions(await tilingSearch(search));
          } finally {
            setSearchLoading(false);
          }
        }
      }
    }, 100),
    [],
  );

  const onChange = useCallback(
    (_: Event, textSearchItem: TextSearchItem | undefined) => {
      const canvas = canvasRef.current;
      if (canvas) {
        clearCanvas(canvas, true);
        if (textSearchItem) {
          setTiling(canvas, textSearchItem.id);
        }
      }
    },
    [],
  );

  return (
    <>
      <h1 className={styles.title}>Tilings</h1>
      <Autocomplete
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
        style={{ width: 300 }}
      />
      <Canvas
        ref={canvasRef}
        height={canvasHeight}
        width={canvasWidth}
        onMouseMove={handleEvent}
        onTouchMove={handleEvent}
      />
    </>
  );
};
