# 15. Packet Diagram

## 15.1. Packet Beta (Short)

~~~mermaid
packet-beta
0-15: "source hash"
16-31: "theme"
32-63: "renderer profile"
~~~

## 15.2. Packet (Full TCP)

~~~mermaid
---
title: "TCP Packet"
---
packet
0-15: "Source Port"
16-31: "Destination Port"
32-63: "Sequence Number"
64-95: "Acknowledgment Number"
96-99: "Data Offset"
100-105: "Reserved"
106: "URG"
107: "ACK"
108: "PSH"
109: "RST"
110: "SYN"
111: "FIN"
112-127: "Window"
128-143: "Checksum"
144-159: "Urgent Pointer"
160-191: "(Options and Padding)"
192-255: "Data (variable length)"
~~~

<!-- katana-mermaid-official:start -->

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 15. Packet Diagram](../official/15-packet.png)

<!-- katana-mermaid-official:end -->
