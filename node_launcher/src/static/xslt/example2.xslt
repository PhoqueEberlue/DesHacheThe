<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">

    <!-- Template to match the root element -->
    <xsl:template match="/">
        <html>
            <head>
                <title>System Information</title>
                <script src="../SaxonJS/SaxonJS2.js"></script>
            </head>
            <body>
                <h1>System Information</h1>

                <xsl:for-each select="//record">
                    <!-- Start of record -->
                    <h2>CPUs</h2>
                    <ul>
                        <!-- Calculate average, minimum, and maximum CPU usage -->
                        <xsl:variable name="cpuNodes" select="cpus/cpu" />
                        <xsl:variable name="cpuCount" select="count($cpuNodes)" />
                        <xsl:variable name="totalCpuUsage" select="sum($cpuNodes/@usage)" />
                        <xsl:variable name="avgCpuUsage" select="$totalCpuUsage div $cpuCount" />

                        <!-- Calculate average, minimum, and maximum CPU frequency -->
                        <xsl:variable name="totalCpuFrequency" select="sum($cpuNodes/@frequency)" />
                        <xsl:variable name="avgCpuFrequency"
                            select="$totalCpuFrequency div $cpuCount" />
                        <li>Average CPU Usage: <xsl:value-of select="$avgCpuUsage" />%</li>
                        <li>Average CPU Frequency: <xsl:value-of select="$avgCpuFrequency" /> MHz</li>
                    </ul>

                    <h2>
        RAM</h2>
                    <p>Total: <xsl:value-of select=".//ram/@total" /> bytes</p>
                    <p>Used: <xsl:value-of
                            select=".//ram/@used" /> bytes</p>
                    <p>Total Swap: <xsl:value-of
                            select=".//ram/@total_swap" /> bytes</p>
                    <p>Used Swap: <xsl:value-of
                            select=".//ram/@used_swap" /> bytes</p>

                    <h2>Disks</h2>
                    <ul>
                        <!-- Calculate total disk space, minimum and maximum available space -->
                        <xsl:variable name="disks" select="disks/disk" />
                        <xsl:variable name="totalDiskSpace" select="sum($disks/@total_space)" />
                        <li>Total Disk Space: <xsl:value-of select="$totalDiskSpace" /> bytes</li>
                    </ul>

                    <h2>
        Networks</h2>
                    <ul>
                        <xsl:for-each select=".//network">
                            <li>
                                <xsl:value-of select="@name" />: Received <xsl:value-of
                                    select="@received" /> bytes, Total Received <xsl:value-of
                                    select="@total_received" /> bytes, Transmitted <xsl:value-of
                                    select="@transmitted" /> bytes, Total Transmitted <xsl:value-of
                                    select="@total_transmitted" /> bytes </li>
                        </xsl:for-each>
                    </ul>

                    <hr />
                    <!-- Separator between records -->
                </xsl:for-each>

            </body>
        </html>
    </xsl:template>

</xsl:stylesheet>