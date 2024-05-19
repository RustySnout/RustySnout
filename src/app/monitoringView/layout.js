'use client';
import React, { useState, useEffect } from 'react';
import Table from "../genericTable/layout";
import Graph from "../graph/layout";
import ControlPanel from '../ControlPanel/layout'; // Correct the import path
import styles from './layout.module.css';
import { invoke } from "@tauri-apps/api/tauri";
import Image from "next/image";

const MonitoringView = () => {
  const [colHeaders, setColHeaders] = useState([]);
  const [rowData, setRowData] = useState([]);
  const [invokeFunction, setInvokeFunction] = useState("get_process_wrapper");
  const [refresh, setRefresh] = useState(false);  
  const [showControlPanel, setShowControlPanel] = useState(false);

  // Button handlers that change the state of invokeFunction to the tauri command we need
  // These are called onClick in the html below
  const handleProcess = () => {
    setInvokeFunction("get_process_wrapper");
  };

  const handleConnection = () => { 
    setInvokeFunction("get_connections_wrapper");
  };

  const handleRemoteAddr = () => { 
    setInvokeFunction("get_remote_address_wrapper");
  };

  const handleControl = () => {
    setShowControlPanel(!showControlPanel);
  };

  useEffect(() => {
    if (!showControlPanel) { // Fetch data only if ControlPanel is not shown
      setRefresh(true);
      const fetchData = async () => {
        invoke(invokeFunction).then((res) => {
          setRefresh(false);
          console.log(res);
          const data = JSON.parse(res);
          setColHeaders(Object.keys(data[0]));
          setRowData(data);
        }).catch((err) => {
          setRefresh(false);
          console.log(err);
        });
      };

      // Fetch data every 5 seconds
      fetchData();
      const intervalId = setInterval(fetchData, 2000);
      return () => clearInterval(intervalId);
    }
  }, [invokeFunction]);

  // This changes the style of the refresh icon to show that the data is being refreshed
  // See layout.module.css for the styles .refreshing and .refresh to see how it is done
  const refreshStyle = refresh ? styles.refreshing : styles.refresh;

  const controlPanelOrTable = showControlPanel ? 
  <ControlPanel /> :  
  <>
    <Image src="/loading.svg" alt="Next.js Logo" width={50} height={50} className={refreshStyle}/>
    <Table rows={rowData} columns={colHeaders} />
  </>;


  return (
    <div className={styles.monitoringBody}>
      <div className={styles.topContainer}>
        <div className={styles.buttonContainer}>
          <button onClick={handleProcess} disabled={showControlPanel} > 
            <Image src="/process.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>  
          </button>
          <button onClick={handleConnection} disabled={showControlPanel} > 
            <Image src="/connections.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>  
          </button>
          <button onClick={handleRemoteAddr} disabled={showControlPanel}> 
            <Image src="/remoteAddr.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/>
          </button>
          <button onClick={handleControl}> 
            <Image src="/control.svg" alt="Next.js Logo" width={50} height={50} className={styles.symbol}/> 
          </button>
        </div>
        <Graph />       
      </div>
      <div className={styles.refreshTable}>
        {controlPanelOrTable}
      </div>
    </div>
  );
};

export default MonitoringView;
